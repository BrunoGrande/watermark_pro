# **Watermark Pro 🌊**

**Watermark Pro** is a high-performance, GPU-accelerated desktop application for batch watermarking images. Built with Rust, it is designed to handle everything from standard JPEGs to massive RAW camera files without breaking a sweat.

## **🚀 Features**

* **Zero-Lag Preview:** Uses the GPU to render the watermark overlay in real-time. Moving sliders feels instant, even on 4K monitors.  
* **True RAW Support:** Native support for professional camera formats (.NEF, .CR2, .ARW, .DNG, .ORF, etc.) using high-fidelity 16-bit decoding.  
* **Batch Processing:** Drag and drop a folder to process thousands of images in seconds.  
* **SIMD Accelerated:** Uses CPU vector instructions (AVX2/SSE) for blazing fast export speeds (up to 10x faster than standard resize algorithms).  
* **High Quality Output:** Uses **Lanczos3** resampling for the sharpest possible watermarks.  
* **Drag & Drop:** Simply drop an image or a folder into the window to get started.

## **🛠️ Tech Stack**

Built with ❤️ in **Rust**.

* **GUI:** [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) (Immediate Mode GUI)  
* **Image Processing:** [image](https://github.com/image-rs/image)  
* **Optimization:** [fast\_image\_resize](https://github.com/Cykooz/fast_image_resize) (SIMD)  
* **RAW Decoding:** [rawloader](https://github.com/pedrocr/rawloader)

## **📦 Installation & Usage**

### **1\. Download (Pre-built)**

If you have compiled the .exe, simply double-click watermark\_pro.exe. No installation required.

### **2\. Build from Source**

You will need [Rust](https://www.rust-lang.org/tools/install) installed.

\# Clone the repository  
'''
git clonehttps://github.com/yourusername/watermark\_pro.git
'''
cd watermark\_pro

\# Run in Release mode (Critical for image processing speed\!)  
cargo run \--release

### **3\. Creating a Standalone Executable**

To generate a standalone .exe file for Windows that hides the console window:

cargo build \--release

You will find the executable in target/release/watermark\_pro.exe.

## **🎮 How to Use**

1. **Input:** Click "Select Input Image" or drag a file/folder onto the window.  
2. **Watermark:** Select your PNG/Logo to use as the watermark.  
3. **Adjust:** Use the sliders to change **Opacity**, **Scale**, and **Position**.  
4. **Process:** Click "Process Images" to save the result. All files are exported as high-quality PNGs.

## **📸 Supported Formats**

* **Standard:** PNG, JPG, JPEG, WEBP  
* **RAW:** NEF (Nikon), CR2 (Canon), ARW (Sony), DNG (Adobe), ORF (Olympus), RW2 (Panasonic), RAF (Fujifilm)

## **📄 License**

MIT License. Feel free to use and modify\!
