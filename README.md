# bevy fluid sample

This project is a fluid simulation sample built using the Bevy engine.

## Features
Currently, this project provides invicid, incompressible 2D fluid simulation with free boundary condition (pressure equals to zero at the boundary) only.

## Getting Started

To run the Bevy Fluid Sample, you will need to have Rust and Cargo installed on your system. Follow these steps to get up and running:

1. Clone the repository:
```ps1
git clone git@github.com:narasan49/bevy-fluid-sample.git
```

2. Navigate to the project directory:

```ps1
cd bevy-fluid-sample
```

3. Build and run the project:

```ps1
cargo run
```

## Acknowledgments
The simulation is inspired by and based on the algorithms described in these books:

- [Fluid Simulation for Computer Graphics](https://www.amazon.co.jp/dp/1482232839) by Robert Bridson
- [GPU Gems Chapter 38](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu) by Mark J. Harris