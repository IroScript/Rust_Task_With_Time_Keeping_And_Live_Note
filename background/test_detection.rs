// Quick test of GPU detection logic
use wgpu;

fn detect_gpu_availability() -> bool {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    
    let mut has_hardware_gpu = false;
    let mut has_basic_render_driver = false;
    
    for adapter in adapters {
        let info = adapter.get_info();
        
        println!("Adapter: {}", info.name);
        println!("  Type: {:?}", info.device_type);
        println!("  Backend: {:?}", info.backend);
        
        if info.name.contains("Basic Render Driver") {
            has_basic_render_driver = true;
            println!("  ⚠ Basic Render Driver detected!");
        }
        
        if info.device_type != wgpu::DeviceType::Cpu 
            && !info.name.contains("Basic Render Driver")
            && !info.name.contains("Software") {
            println!("  ✓ Hardware GPU");
            has_hardware_gpu = true;
        } else {
            println!("  ✗ Software renderer");
        }
        println!();
    }
    
    println!("=== RESULT ===");
    println!("has_hardware_gpu: {}", has_hardware_gpu);
    println!("has_basic_render_driver: {}", has_basic_render_driver);
    
    if has_basic_render_driver && !has_hardware_gpu {
        println!("❌ CRITICAL: Only Microsoft Basic Render Driver available!");
        println!("   Bloom should be DISABLED");
    }
    
    has_hardware_gpu
}

fn main() {
    println!("=== GPU Detection Test ===\n");
    
    let has_gpu = detect_gpu_availability();
    
    println!("\n=== QUALITY SETTINGS ===");
    if has_gpu {
        println!("Mode: HIGH (GPU available)");
        println!("Particles: 800");
        println!("Bloom: ENABLED");
    } else {
        println!("Mode: LOW (Software renderer)");
        println!("Particles: 10");
        println!("Bloom: DISABLED");
    }
}