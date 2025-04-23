// src/bin/test_gpu.rs
use tch::Device;

fn main() {
    println!("Detected device: {:?}", Device::cuda_if_available());
}
