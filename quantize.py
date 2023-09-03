from PIL import Image

from transformers import DPTForDepthEstimation, DPTFeatureExtractor
import torch
model = DPTForDepthEstimation.from_pretrained("Intel/dpt-hybrid-midas", low_cpu_mem_usage=True)
feature_extractor = DPTFeatureExtractor.from_pretrained("Intel/dpt-hybrid-midas")
%pip install accelerate
model = DPTForDepthEstimation.from_pretrained("Intel/dpt-hybrid-midas", low_cpu_mem_usage=True)
feature_extractor = DPTFeatureExtractor.from_pretrained("Intel/dpt-hybrid-midas")
model = DPTForDepthEstimation.from_pretrained("Intel/dpt-hybrid-midas")
feature_extractor = DPTFeatureExtractor.from_pretrained("Intel/dpt-hybrid-midas")
image = Image.open("/home/joseph/Pictures/KoyWZ74.jpeg")
features = feature_extractor(image=image, return_tensors="pt")
features = feature_extractor(images=image, return_tensors="pt")
features.shape
features?
features.keys()
features['pixel_values'].shape
feature_extractor?
features['pixel_values'].max()
features['pixel_values'].min()
with torch.no_grad():
    outputs = model(pixel_values=features['pixel_values])
with torch.no_grad():
    outputs = model(pixel_values=features['pixel_values'])
    predicted_depth = outputs.predicted_depth
predicted_depth?
predicted_depth.shape
model
input_names = ['pixel_values']
output_names = ['predicted_depth']
dummy_input = torch.rand((3, 384, 384))
dummy_input.min()
dummy_input.max()
dummy_input = torch.rand((3, 384, 384)) * 2.0 - 1.0
dummy_input.min()
dummy_input.max()
out = model(pixel_values=dummy_input)
dummy_input = torch.rand((1, 3, 384, 384)) * 2.0 - 1.0
out = model(pixel_values=dummy_input)
torch.onnx.export(model, dummy_input, "midas_hybrid_384_384.onnx", verbose=True, input_names=input_names, output_names=output_names)
%pip install onnx
torch.onnx.export(model, dummy_input, "midas_hybrid_384_384.onnx", verbose=True, input_names=input_names, output_names=output_names)
import onnx
model2 = onnx.load("./midas_hybrid_384_384.onnx")
import onnxruntime as ort
%pip install onnxruntime
del model2
import onnxruntime as ort
inf_ses = ort.InferenceSession("./midas_hybrid_384_384.onnx")
depth = inf_ses.run?
depth = inf_ses.run(None, {"pixel_values": dummy_input})
depth = inf_ses.run(None, dummy_input)
depth = inf_ses.run(None, [dummy_input,])
import numpy
depth = inf_ses.run(None, numpy.random.rand((1, 3 ,384, 384)))
depth = inf_ses.run(None, numpy.random.random((1, 3, 384, 384)))
depth = inf_ses.run(numpy.random.random((1, 3, 384, 384)))
depth = inf_ses.run(None, [numpy.random.random((1, 3, 384, 384))])
depth = inf_ses.run(None, {'pixel_values':numpy.random.random((1, 3, 384, 384))})
depth = inf_ses.run(None, {'pixel_values':numpy.random.random((1, 3, 384, 384)).astype(numpy.float)})
depth = inf_ses.run(None, {'pixel_values':numpy.random.random((1, 3, 384, 384)).astype(float)})
depth = inf_ses.run(None, {'pixel_values':numpy.random.random((1, 3, 384, 384)).astype(numpy.float32)})
depth.shape
depth[0].shape
from onnxruntime.quantization import quantize_dynamic, QuantType
model_filename_fp32 = "./midas_hybrid_384_384.onnx"
model_out_quant = "./midas_hybrid_384_384_quant.onnx"
quantized_model = quantize_dynamic(model_filename_fp32, model_out_quant)
depth_original = inf_ses.run(None, {'pixel_values':numpy.random.random((1, 3, 384, 384)).astype(numpy.float32)})
dummy_numpy = numpy.random.random((1, 3, 384, 384)).astype(numpy.float32)
depth_original = inf_ses.run(None, {'pixel_values':dummy_numpy})
inf_ses_quant = ort.InferenceSession("./midas_hybrid_384_384_quant.onnx")
history -f ./quantize.py

"""
    DPTFeatureExtractor {
           "do_normalize": true,
           "do_rescale": true,
           "do_resize": true,
           "ensur <...> ,
           "rescale_factor": 0.00392156862745098,
           "size": {
           "height": 384,
           "width": 384
           }
           }
           
File:            ~/.virtualenvs/motion_capture_mk5/lib/python3.10/site-packages/transformers/models/dpt/feature_extraction_dpt.py
Docstring:       <no docstring>
Class docstring:
Constructs a DPT image processor.

Args:
    do_resize (`bool`, *optional*, defaults to `True`):
        Whether to resize the image's (height, width) dimensions. Can be overidden by `do_resize` in `preprocess`.
    size (`Dict[str, int]` *optional*, defaults to `{"height": 384, "width": 384}`):
        Size of the image after resizing. Can be overidden by `size` in `preprocess`.
    keep_aspect_ratio (`bool`, *optional*, defaults to `False`):
        If `True`, the image is resized to the largest possible size such that the aspect ratio is preserved. Can
        be overidden by `keep_aspect_ratio` in `preprocess`.
    ensure_multiple_of (`int`, *optional*, defaults to 1):
        If `do_resize` is `True`, the image is resized to a size that is a multiple of this value. Can be overidden
        by `ensure_multiple_of` in `preprocess`.
    resample (`PILImageResampling`, *optional*, defaults to `PILImageResampling.BILINEAR`):
        Defines the resampling filter to use if resizing the image. Can be overidden by `resample` in `preprocess`.
    do_rescale (`bool`, *optional*, defaults to `True`):
        Whether to rescale the image by the specified scale `rescale_factor`. Can be overidden by `do_rescale` in
        `preprocess`.
    rescale_factor (`int` or `float`, *optional*, defaults to `1/255`):
        Scale factor to use if rescaling the image. Can be overidden by `rescale_factor` in `preprocess`.
    do_normalize (`bool`, *optional*, defaults to `True`):
        Whether to normalize the image. Can be overridden by the `do_normalize` parameter in the `preprocess`
        method.
    image_mean (`float` or `List[float]`, *optional*, defaults to `IMAGENET_STANDARD_MEAN`):
        Mean to use if normalizing the image. This is a float or list of floats the length of the number of
        channels in the image. Can be overridden by the `image_mean` parameter in the `preprocess` method.
    image_std (`float` or `List[float]`, *optional*, defaults to `IMAGENET_STANDARD_STD`):
        Standard deviation to use if normalizing the image. This is a float or list of floats the length of the
        number of channels in the image. Can be overridden by the `image_std` parameter in the `preprocess` method.
Init docstring:  Set elements of `kwargs` as attributes.
Call docstring:  Preprocess an image or a batch of images.

In [16]: features['pixel_values'].max()
Out[16]: tensor(0.9686)

In [17]: features['pixel_values'].min()
Out[17]: tensor(-0.9922)

"""