enum ImageFormat { jpeg, png }

class CompressionConfig {
  final int quality;          // 0-100质量等级
  final int? targetSizeKb;    // 目标文件大小（可选）
  final ImageFormat format;   // 输出格式

  const CompressionConfig({
    this.quality = 80,
    this.targetSizeKb,
    this.format = ImageFormat.jpeg,
  }) : assert(quality >= 0 && quality <= 100);
}