"""
Custom SAHI DetectionModel implementation for SAMDEF YOLO ONNX Model (run15_FP16.onnx).
Runs native FP16 Tensor Core inference on CUDA via ONNX Runtime.
"""

from typing import Any, List, Optional
import numpy as np
import onnxruntime as ort
from sahi.models.base import DetectionModel
from sahi.prediction import ObjectPrediction
from sahi.utils.compatibility import fix_full_shape_list, fix_shift_amount_list

CATEGORY_MAPPING = {
    "0": "Light-Vehicle",
    "1": "Boxy-Truck",
    "2": "Long-Trucks",
    "3": "Small-Boat",
    "4": "Large-Ship",
    "5": "Fixed-Wing",
    "6": "Helicopter",
    "7": "Building",
    "8": "Other-Structure",
    "9": "Storage-Tank",
    "10": "Container-Lot",
    "11": "Construction-Site",
    "12": "Railway",
    "13": "Engineering-Machinery",
    "14": "Tower-Pylon",
}


class SamdefYoloOnnxDetectionModel(DetectionModel):
    """
    SAHI DetectionModel wrapper for SAMDEF YOLO ONNX models with fused outputs [1, max_det, 6].
    """

    def __init__(
        self,
        model_path: str,
        confidence_threshold: float = 0.01,
        device: str = "cuda:0",
        category_mapping: Optional[dict] = None,
        load_at_init: bool = True,
        image_size: int = 1152,
        **kwargs,
    ):
        self.image_size = image_size
        self.providers = [
            (
                "CUDAExecutionProvider",
                {
                    "device_id": 0,
                    "arena_extend_strategy": "kNextPowerOfTwo",
                    "gpu_mem_limit": 6 * 1024 * 1024 * 1024,
                    "cudnn_conv_algo_search": "EXHAUSTIVE",
                    "do_copy_in_default_stream": True,
                },
            ),
            "CPUExecutionProvider",
        ]
        super().__init__(
            model_path=model_path,
            confidence_threshold=confidence_threshold,
            device=device,
            category_mapping=category_mapping or CATEGORY_MAPPING,
            load_at_init=load_at_init,
            image_size=image_size,
            **kwargs,
        )

    def check_dependencies(self):
        try:
            import onnxruntime  # noqa: F401
        except ImportError:
            raise ImportError(
                "onnxruntime or onnxruntime-gpu is required. Install via pip install onnxruntime-gpu"
            )

    def load_model(self):
        opts = ort.SessionOptions()
        opts.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        opts.log_severity_level = 3  # Suppress harmless plugin EP lookup warnings
        self.model = ort.InferenceSession(
            self.model_path,
            sess_options=opts,
            providers=self.providers,
        )

        self.input_name = self.model.get_inputs()[0].name
        self.output_name = self.model.get_outputs()[0].name

    def perform_inference(self, image: np.ndarray):
        """
        Input: image as HWC RGB numpy array (e.g. 1152x1152x3).
        """
        self._original_shapes = [image.shape]
        h, w, c = image.shape
        # Ensure slice matches model input dimensions (1152x1152)
        if h != self.image_size or w != self.image_size:
            import cv2
            image = cv2.resize(image, (self.image_size, self.image_size))

        # Planar transposition (HWC to CHW) and normalization (0.0 to 1.0)
        tensor = np.transpose(image, (2, 0, 1)).astype(np.float32) / 255.0
        tensor = np.expand_dims(tensor, axis=0)  # [1, 3, 1152, 1152]

        # Execute on CUDA
        outputs = self.model.run([self.output_name], {self.input_name: tensor})
        self._original_predictions = outputs[0]  # Shape: [1, 5000, 6]

    def perform_batch_inference(self, images: List[np.ndarray]):
        """
        Input: list of images as HWC RGB numpy arrays.
        Batches images into [B, 3, 1152, 1152] and runs in a single GPU pass.
        """
        self._batch_images = None  # Prevents SAHI from falling back to sequential single-slice loop
        self._original_shapes = [img.shape for img in images]
        processed_tensors = []
        for img in images:
            h, w, c = img.shape
            if h != self.image_size or w != self.image_size:
                import cv2
                img = cv2.resize(img, (self.image_size, self.image_size))
            tensor = np.transpose(img, (2, 0, 1)).astype(np.float32) / 255.0
            processed_tensors.append(tensor)
        batch_tensor = np.stack(processed_tensors, axis=0)  # Shape: [B, 3, 1152, 1152]

        # Execute batched inference on CUDA
        outputs = self.model.run([self.output_name], {self.input_name: batch_tensor})
        self._original_predictions = outputs[0]  # Shape: [B, 5000, 6]


    def _create_object_prediction_list_from_original_predictions(
        self,
        shift_amount_list: Optional[List[List[float]]] = None,
        full_shape_list: Optional[List[List[float]]] = None,
    ):
        """
        Convert raw [1, 5000, 6] detections into SAHI ObjectPrediction list.
        """
        original_predictions = self._original_predictions
        if original_predictions is None:
            self._object_prediction_list_per_image = [[]]
            return

        shift_amount_list_typed = fix_shift_amount_list(shift_amount_list)
        full_shape_list_typed = fix_full_shape_list(full_shape_list)

        object_prediction_list_per_image = []

        for image_ind, image_predictions in enumerate(original_predictions):
            shift_amount = [int(x) for x in shift_amount_list_typed[image_ind]]
            full_shape = (
                None
                if full_shape_list_typed is None
                else [int(x) for x in full_shape_list_typed[image_ind]]
            )
            image_shape = self._original_shapes[image_ind]
            object_prediction_list = []

            # Filter by confidence threshold
            # Format: [x1, y1, x2, y2, score, class_id]
            valid_mask = image_predictions[:, 4] > self.confidence_threshold
            proposals = image_predictions[valid_mask]

            for prop in proposals:
                x1, y1, x2, y2, score, class_id = prop
                cid = int(class_id)
                category_name = self.category_mapping.get(str(cid), f"class_{cid}")

                # Clamp coordinates to tile boundary
                x1 = max(0.0, float(x1))
                y1 = max(0.0, float(y1))
                x2 = min(float(image_shape[1]), float(x2))
                y2 = min(float(image_shape[0]), float(y2))

                if x2 <= x1 or y2 <= y1:
                    continue

                prediction = ObjectPrediction(
                    bbox=[x1, y1, x2, y2],
                    category_id=cid,
                    category_name=category_name,
                    score=float(score),
                    shift_amount=shift_amount,
                    full_shape=list(image_shape[:2]) if full_shape is None else full_shape,
                )
                object_prediction_list.append(prediction)

            object_prediction_list_per_image.append(object_prediction_list)

        self._object_prediction_list_per_image = object_prediction_list_per_image

