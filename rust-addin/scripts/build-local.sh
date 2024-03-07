#!/usr/bin/env bash
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


set -e

cd "$(dirname "${BASH_SOURCE[0]}")"/..

# Windows
rustup target add x86_64-pc-windows-gnu --toolchain stable
rustup run stable cargo rustc --release --target=x86_64-pc-windows-gnu -- -C linker=x86_64-w64-mingw32-gcc

# Linux
rustup target add x86_64-unknown-linux-gnu --toolchain stable
rustup run stable cargo rustc --release --target=x86_64-unknown-linux-gnu -- -C linker=x86_64-unknown-linux-gnu-gcc