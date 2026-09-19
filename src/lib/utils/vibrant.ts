import { BasicPipeline, Vibrant } from "@vibrant/core";
import { BrowserImage } from "@vibrant/image-browser";
import { MMCQ } from "@vibrant/quantizer-mmcq";
import { DefaultGenerator } from "@vibrant/generator-default";

Vibrant.DefaultOpts.ImageClass = BrowserImage;
Vibrant.DefaultOpts.quantizer = "mmcq";
Vibrant.DefaultOpts.generators = ["default"];
Vibrant.DefaultOpts.filters = ["default"];
Vibrant.use(new BasicPipeline()
	.filter.register("default", (red, green, blue, alpha) => alpha >= 125 && !(red > 250 && green > 250 && blue > 250))
	.quantizer.register("mmcq", MMCQ)
	.generator.register("default", DefaultGenerator));

export { Vibrant };
