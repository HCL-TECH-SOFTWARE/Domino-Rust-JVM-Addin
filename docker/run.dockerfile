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

ARG BASEIMAGE=hclcom/domino:14.0

FROM --platform=amd64 ${BASEIMAGE}

ENV NOTESBIN="/opt/hcl/domino/notes/latest/linux"
ENV DEBUG="true"
ENV LANG="en_US.UTF-8"
ENV SetupAutoConfigure="1"
ENV SetupAutoConfigureParams="/local/domino-config.json"
ENV DOMINO_DOCKER_STDOUT="yes"

# Copy in the built addin and JAR
USER root
COPY addin-dist/target/dependency/rustaddin-lib /opt/hcl/domino/notes/latest/linux/rustaddin-lib
COPY addin-dist/target/dependency/rust-addin-linux-x64.bin /opt/hcl/domino/notes/latest/linux/rustaddin
USER notes

COPY --chown=notes:notes docker/domino-config.json /local/
COPY --chown=notes:notes docker/AddinJavaOptionsFile.txt /local/
