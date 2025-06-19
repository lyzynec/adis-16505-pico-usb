#!/usr/bin/bash

SCRIPTPATH=$(realpath $0)
DIRPATH=$(dirname $SCRIPTPATH)

git submodule init
git submodule update

# IMU

echo "This script requires having rust, cargo and python installed."

sudo apt install -y git libclang-dev python3-pip python3-vcstool
cargo install --debug cargo-ament-build
pip3 install git+https://github.com/colcon/colcon-cargo.git
pip3 install git+https://github.com/colcon/colcon-ros-cargo.git

vcs import $DIRPATH/src < $DIRPATH/src/ros2_rust/ros2_rust_humble.repos
