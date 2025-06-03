use image::{ImageFormat, DynamicImage, ImageReader};
use std::{path::Path, fs};
use serde::{Deserialize, Serialize};
use sysinfo::{System,};
use rayon::prelude::*;
use std::sync::Mutex;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_path: String,
    pub compressed_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
/// 批处理压缩函数
pub fn compress_batch(
    file_paths: Vec<String>,
    quality: u8,
    target_size_kb: Option<u32>,
    format: String,
) -> Vec<Result<CompressionResult, String>> {
    // 获取系统信息以动态调整并行度
    let sys = System::new_all();

    // 根据可用内存和CPU核心数计算并行度
    let available_memory_mb = sys.available_memory() / (1024 * 1024);
    let cpu_cores = sys.cpus().len();

    // 计算并行度 - 保守策略，避免内存不足
    let parallelism = calculate_parallelism(available_memory_mb, cpu_cores);

    // 创建线程池
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build()
        .expect("Failed to build thread pool");

    // 使用互斥锁保护共享资源（如果需要）
    let error_count = Mutex::new(0);

    // 并行处理所有文件
    pool.install(|| {
        file_paths
            .into_par_iter()
            .map(|file_path| {
                // 检查错误计数，如果太多错误则提前终止
                let errors = error_count.lock().unwrap();
                if *errors > 10 {
                    return Err("Too many errors, stopping early".to_string());
                }
                drop(errors); // 显式释放锁

                // 处理单个文件
                match compress_single(file_path, quality,  "png".parse().unwrap()) {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        // 更新错误计数
                        let mut errors = error_count.lock().unwrap();
                        *errors += 1;
                        Err(e)
                    }
                }
            })
            .collect()
    })
}

/// 根据系统资源计算并行度
fn calculate_parallelism(available_memory_mb: u64, cpu_cores: usize) -> usize {
    // 基础并行度 - 不超过CPU核心数
    let mut parallelism = cpu_cores.min(8); // 最多8线程

    // 根据可用内存调整
    if available_memory_mb < 512 {
        parallelism = parallelism.min(2); // 低内存设备
    } else if available_memory_mb < 1024 {
        parallelism = parallelism.min(4); // 中等内存设备
    }

    // 确保至少1个线程
    parallelism.max(1)
}
pub fn compress_single(
    file_path: String,
    quality: u8,
    format: String,
) -> Result<CompressionResult, String> {
    let img = load_image(&file_path)?;
    let original_size = std::fs::metadata(&file_path)
        .map_err(|e| format!("获取文件元数据失败: {}", e))?
        .len();

    let output_path = generate_output_path(&file_path, &format);
    let compressed_size = match format.as_str() {
        "jpeg" => save_jpeg(&img, &output_path, quality)?,
        "png" => save_png(&img, &output_path)?,
        _ => return Err("不支持的图像格式".into()),
    };

    Ok(CompressionResult {
        original_path: file_path.clone(),
        compressed_path: output_path,
        original_size,
        compressed_size,
    })
}

fn load_image(path: &str) -> Result<DynamicImage, String> {
    // 1. 检查文件是否存在且非空
    let metadata = fs::metadata(path)
        .map_err(|e| format!("文件检查失败: {}", e))?;

    if metadata.len() == 0 {
        return Err("文件为空".into());
    }

    // 2. 使用 Reader 自动检测格式
    let reader = ImageReader::open(path)
        .map_err(|e| format!("打开文件失败: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("检测格式失败: {}", e))?;

    // 3. 尝试解码
    match reader.decode() {
        Ok(img) => Ok(img),
        Err(orig_err) => {
            // 4. 如果失败，尝试读取原始字节
            let data = fs::read(path)
                .map_err(|e| format!("读取文件失败: {}", e))?;

            // 5. 尝试修复常见的图像格式问题
            if data.starts_with(b"\xFF\xD8") {
                // JPEG 文件
                image::load_from_memory(&data)
            } else if data.starts_with(b"\x89PNG") {
                // PNG 文件
                image::load_from_memory(&data)
            } else if data.starts_with(b"GIF8") {
                // GIF 文件
                image::load_from_memory(&data)
            } else {
                // 尝试所有支持的格式
                image::load_from_memory(&data)
            }
                .map_err(|e| {
                    format!(
                        "原始错误: {}, 修复尝试错误: {}",
                        orig_err, e
                    )
                })
        }
    }
}

fn generate_output_path(original: &str, format: &str) -> String {
    use std::path::{Path, PathBuf};

    let original_path = Path::new(original);
    let file_stem = original_path
        .file_stem() // 获取文件名（不带扩展名）
        .unwrap_or_else(|| std::ffi::OsStr::new("compressed")) // 异常时使用默认名
        .to_string_lossy();

    // 构造新路径：原路径目录 + 文件名_compressed.新扩展名
    let mut output_path = PathBuf::from(original_path.parent().unwrap_or(Path::new(".")));
    output_path.push(format!("{}_compressed.{}", file_stem, format));

    output_path.to_string_lossy().into_owned()
}

fn save_jpeg(img: &DynamicImage, path: &str, quality: u8) -> Result<u64, String> {
    use std::fs::File;

    // 创建输出文件
    let mut file = File::create(path).map_err(|e| format!("创建 JPEG 文件失败: {}", e))?;

    // 使用 JPEG 编码器（质量参数 0-100）
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
    encoder
        .encode_image(img)
        .map_err(|e| format!("编码 JPEG 失败: {}", e))?;

    // 获取压缩后的文件大小
    Ok(file
        .metadata()
        .map_err(|e| format!("获取 JPEG 文件大小失败: {}", e))?
        .len())
}

fn save_png(img: &DynamicImage, path: &str) -> Result<u64, String> {
    use std::fs::File;

    // 创建输出文件
    let mut file = File::create(path)
        .map_err(|e| format!("创建 PNG 文件失败: {}", e))?;

    // 使用 PNG 编码器（默认压缩级别）
    img.write_to(&mut file, ImageFormat::Png)
        .map_err(|e| format!("编码 PNG 失败: {}", e))?;

    // 获取压缩后的文件大小
    Ok(file.metadata()
        .map_err(|e| format!("获取 PNG 文件大小失败: {}", e))?
        .len())
}