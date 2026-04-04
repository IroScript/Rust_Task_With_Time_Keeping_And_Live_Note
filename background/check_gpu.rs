// Simple GPU detection utility
// Run with: cargo run --bin check_gpu

fn main() {
    println!("=== GPU Detection Utility ===\n");
    
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    
    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    let mut adapter_count = 0;
    let mut has_hardware = false;
    
    for adapter in adapters {
        adapter_count += 1;
        let info = adapter.get_info();
        
        println!("Adapter #{}:", adapter_count);
        println!("  Name: {}", info.name);
        println!("  Backend: {:?}", info.backend);
        println!("  Device Type: {:?}", info.device_type);
        println!("  Vendor: {}", info.vendor);
        println!("  Device: {}", info.device);
        
        if info.device_type != wgpu::DeviceType::Cpu {
            has_hardware = true;
            println!("  ✓ This is a HARDWARE GPU");
        } else {
            println!("  ✗ This is a SOFTWARE renderer (CPU-based)");
        }
        
        if info.name.contains("Basic Render Driver") {
            println!("  ⚠ WARNING: Microsoft Basic Render Driver detected!");
            println!("     This is a fallback software renderer with severe limitations.");
            println!("     3D applications may crash or perform very poorly.");
        }
        
        println!();
    }
    
    println!("=== Summary ===");
    println!("Total adapters found: {}", adapter_count);
    
    if adapter_count == 0 {
        println!("❌ NO GPU ADAPTERS FOUND!");
        println!("   Your system may not have proper graphics drivers installed.");
    } else if !has_hardware {
        println!("❌ NO HARDWARE GPU DETECTED");
        println!("   Only software renderers available.");
        println!("\n📋 Recommendations:");
        println!("   1. Install/update your GPU drivers");
        println!("   2. Enable integrated graphics in BIOS if available");
        println!("   3. Use a system with a dedicated or integrated GPU");
    } else {
        println!("✓ Hardware GPU available - application should work normally");
    }
}
