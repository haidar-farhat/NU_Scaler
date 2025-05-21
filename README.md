# Nu Scaler

[![Build](https://github.com/haidar-farhat/NU_Scaler/actions/workflows/main.yml/badge.svg)](https://github.com/haidar-farhat/NU_Scaler/actions)
![License](https://img.shields.io/github/license/haidar-farhat/NU_Scaler)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![Last Commit](https://img.shields.io/github/last-commit/haidar-farhat/NU_Scaler)


<img src="./readme/title1.svg"/>

<br><br>

<!-- project overview -->
<img src="./readme/title2.svg"/>
Nu Scaler is a modern desktop application designed to upscale and enhance images and video frames. It aims to provide high-quality results using smart, performance-optimized algorithms that work locally on your machine.

Built with a clean and intuitive interface, Nu Scaler helps users improve visual quality without needing expensive hardware or a constant internet connection. It's especially useful for gamers, streamers, and content creators dealing with low resolution, poor frame rate, or slow internet speeds.

<br><br>

<!-- System Design -->
<img src="./readme/title3.svg"/>

### Architecture Overview

Nu Scaler follows a hybrid architecture combining Python and Rust to balance performance and flexibility. The user interface is built using PySide6 (Qt for Python), offering a modern and responsive cross-platform GUI. Behind the scenes, the heavy lifting is done in Rust, where advanced upscaling and frame interpolation algorithms are executed using WGPU-powered shaders for GPU acceleration. This separation allows the GUI to remain responsive while the computationally intensive tasks are offloaded to efficient, low-level Rust modules, ensuring both speed and stability across different systems.
 
| Component Diagram                       |
| --------------------------------------- |
| ![Landing](./readme/demo/component_diagrame.png) |


| Flow Diagram                          |
| ------------------------------------- |
| ![fsdaf](./readme/demo/flow.png)   |
<br><br>

<!-- Project Highlights -->
<img src="./readme/title4.svg"/>

### NU's Features

| NU's highlight     |
| --------------------------------------- |
| ![Landing](./readme/demo/high.png) | 

- **Frame Interpolation**: Smooths motion in videos, ideal for gaming and streaming.
- **Upscaling**: Fast, high-quality offline upscaling using Rust and WGPU shaders.
- **Cross-Compatibility**: Runs on all platforms with a sleek UI and advanced features.

<br><br>

<!-- Demo -->
<img src="./readme/title5.svg"/>

### Showcase
| Real-Time Test                     |
| ------------------------------------- |
| ![fsdaf](./readme/demo/testrun.gif)   |


| Sample Test                             |Performance Testing                        |
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/stat.gif) | ![fsdaf](./readme/demo/stat.png)   |


### GUI

| Live Feed Main Screen                   | Live Feed (Active)                  |
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/live_main.png) | ![fsdaf](./readme/demo/live_on.png)   |



|  Overlay                                | settings screen                       |
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/Overlay.png)   | ![fsdaf](./readme/demo/settings.png)  |

<br><br>
### Web page


| Admin Main screen                           | manage users screen                   |
| ---------------------------------------     | ------------------------------------- |
| ![Landing](./readme/demo/admin_main.png)    | ![fsdaf](./readme/demo/users.png)     |



<br><br>


<!-- Testing -->
<img src="./readme/title6.svg"/>

### Debug and testing

|  Debug Screen                           | Sample Performance                     |
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/debug.png)     | ![fsdaf](./readme/demo/smpl_pef.png)  |


| Low-Res 2D                              | Enhanced 2D|
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/sprite1.gif)   | ![fsdaf](./readme/demo/sprite2.gif)   |


<br><br>

<!-- Deployment -->
<img src="./readme/title7.svg"/>

| Deployment Worflow                      | Lint workflow                         |
| --------------------------------------- | ------------------------------------- |
| ![Landing](./readme/demo/cicd1.png)     | ![fsdaf](./readme/demo/cicd2.png)     |

### Live Demo

You can try Nu Scaler from the official site:

🌐 **[Live Site](http://15.237.190.24/)** – Desktop app preview and download links.


| Home Screen                             | Download Screen                       | 
| --------------------------------------- | ------------------------------------- | 
| ![Landing](./readme/demo/home.png)      | ![fsdaf](./readme/demo/download.png)  | 

<br><br>
