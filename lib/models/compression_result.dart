class CompressionResult {
  final String originalPath;
  /// 压缩后的文件路径
  final String compressedPath;

  /// 原始文件大小（字节）
  final BigInt originalSize;

  /// 压缩后文件大小（字节）
  final BigInt compressedSize;

  const CompressionResult({
    required this.originalPath,
    required this.compressedPath,
    required this.originalSize,
    required this.compressedSize,
  });

  @override
  String toString() {
    return 'CompressionResult('
        'compressedPath: $compressedPath, '
        'originalSize: $originalSize bytes, '
        'compressedSize: $compressedSize bytes)';
  }
}