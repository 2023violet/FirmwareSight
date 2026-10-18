# Visual report renderer dependency notices

FirmwareSight is licensed under Apache-2.0 (ADR-0035). These notices describe third-party code only; they do not license FirmwareSight. The full third-party list is THIRD_PARTY_LICENSES.md.

## arrayref 0.3.9

Source: https://crates.io/crates/arrayref/0.3.9
Registry checksum and exact dependency version are recorded in Cargo.lock.
License: BSD-2-Clause. The text below is copied verbatim from the fetched crate LICENSE.
The Owner NEXT-45 exception in deny.toml applies only to this package at exactly 0.3.9.

Source redistribution must retain this notice; binary redistribution must include it in accompanying documentation/materials. Distribution is authorized under Apache-2.0 since ADR-0035; THIRD_PARTY_LICENSES.md carries this notice into binary releases. This repository notice must accompany any separately authorized distribution; it does not enter the firmware Release Bundle or change its identity.

```text
Copyright (c) 2015 David Roundy <roundyd@physics.oregonstate.edu>
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the
   distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

## Renderer admission

resvg 0.48.1 (Apache-2.0 OR MIT) rasterizes the same bounded SVG scene used for vector export. Only text and system-fonts are enabled; default raster-image, SVGZ and mmap-font features are disabled. The standard library and existing PNG encoder cannot perform Unicode shaping and SVG rasterization. No network service, fonts, generic shell permission, or GPU dependency is added. This dependency stays in the report/application boundary, outside Core.

Cargo.lock records the resolved transitive graph; online cargo-deny checks licenses, advisories, bans and sources. Local font files are read for rendering and never redistributed. System font variation limits cross-host PNG byte equality. No claim of a complete binary distribution notice set is made by this admission; public distribution remains subject to a complete notice delivery review.
