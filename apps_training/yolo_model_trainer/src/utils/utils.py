import onnx

model_path = "/home/shatadal/SAMDEF/apps_deploy/detector/model/best_fp16.onnx"
patched_path = "/home/shatadal/SAMDEF/apps_deploy/detector/model/best_fp16_patched.onnx"

model = onnx.load(model_path)

for node in model.graph.node:
    if node.op_type == "Cast" and node.name == "graph_output_cast0":
        for attr in node.attribute:
            if attr.name == "to":
                print(f"Changing Cast node {node.name} from {attr.i} to 10 (FLOAT16)")
                attr.i = 10  # 10 = FLOAT16

onnx.save(model, patched_path)
print("Patched model saved as", patched_path)
