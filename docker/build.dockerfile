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

FROM maven:3.9-eclipse-temurin-17

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o install.sh && \
  sh install.sh -y -t x86_64-pc-windows-gnu,x86_64-unknown-linux-gnu
RUN apt update && apt install -y gcc g++-mingw-w64-x86-64 gcc-x86-64-linux-gnu

ENV PATH="/root/.cargo/bin:$PATH"

ENTRYPOINT ["mvn", "clean", "install", "-P", "inner-docker,!build-linux,!build-local"]