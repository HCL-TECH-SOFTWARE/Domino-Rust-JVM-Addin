#
# Copyright (c) 2023-2024 HCL America, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

FROM rust:latest 

# Install x64 GCC tools
RUN apt update && apt upgrade -y
RUN apt install -y g++-mingw-w64-x86-64 gcc-x86-64-linux-gnu

# Ensure that the local native toolchain is present for e.g. ARM hosts,
#   as rustup complains otherwise even if we don't target it
RUN rustup toolchain install stable

# Install our target x64 toolchains to cache at the Docker layer
RUN rustup target add x86_64-unknown-linux-gnu --toolchain stable
RUN rustup target add x86_64-pc-windows-gnu --toolchain stable

# Run the inner build script
WORKDIR /app
CMD ["/usr/bin/bash", "/app/docker/inner-build.sh"]