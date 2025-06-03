import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:rust_image_compress/rust_image_compress.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {

  Future<CompressionResult> compressSingle1(
      File file, {
        CompressionConfig config = const CompressionConfig(
          quality: 30,
          targetSizeKb: 300
        ),
      }) async {
    try {
      final result = await compressSingle(
        filePath: file.path,
        quality: config.quality,
        format: config.format.name,
      );
      return CompressionResult(
        originalPath: file.path,
        compressedPath: result.compressedPath,
        originalSize: result.originalSize,
        compressedSize: result.compressedSize,
      );
    }catch (e) {
      print(e);
      rethrow;
    }
  }

  // 选择图片
  Future<void> _selectImage() async {
    final ImagePicker picker = ImagePicker();
    final XFile? image = await picker.pickImage(source: ImageSource.gallery);
    if (image != null) {

      DateTime startTime = DateTime.now();
      final result = await compressSingle1(File(image.path));
      DateTime endTime = DateTime.now();

      print("原始大小：${formatBytes(await image.length())}");
      print("输出地址：${result.compressedPath }");
      print("压缩大小：${formatBytes(result.compressedSize.toInt())}");
      print("耗时：${endTime.difference(startTime).inMilliseconds}ms");
    }
  }

  // 大小格式化
  String formatBytes(int bytes) {
    if (bytes < 1024) {
      return '${bytes} B';
    } else if (bytes < 1024 * 1024) {
      return '${(bytes / 1024).toStringAsFixed(2)} KB';
    } else if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(2)} MB';
    } else {
      return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
    }
  }


  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
          child: ElevatedButton(onPressed: (){
            _selectImage();
          }, child: Text("Select Image")),
        ),
      ),
    );
  }
}
