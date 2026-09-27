---
title: "Market Source Register"
doc_id: "FS-RSCH-007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-27"
---

# Market Source Register

本文件补足 v0.3.0 市场证据“有结论、回查链接不足”的问题。

检索/复核基线：2026-09-27

| Evidence | Source | Type | Use |
|---|---|---|---|
| Device/build traceability ambiguity | https://www.reddit.com/r/embedded/comments/1uwbbl9/how_do_you_manage_traceabiltity_of_device_state/ | community firsthand discussion | problem signal |
| Legacy STM32 SBOM manual workaround | https://www.reddit.com/r/embedded/comments/1ukhpg6/sbom_software_bill_of_material_generation_for/ | community firsthand discussion | problem signal |
| MCU SBOM tooling/CPE/PURL/VEX friction | https://www.reddit.com/r/embedded/comments/1vffgjo/mcu_firmware_sbom_generation_is_there_an_actual/ | community discussion | problem signal |
| EU CRA overview | https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act | primary government | regulatory context |
| CRA reporting | https://digital-strategy.ec.europa.eu/en/policies/cra-reporting | primary government | dates/obligations |
| Regulation (EU) 2024/2847 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847 | primary law text | regulatory source |
| Electric UI | https://electricui.com/ | vendor | adjacent paid tooling evidence |
| awesome-design-md | https://github.com/VoltAgent/awesome-design-md | upstream repository | design reference |
| AssureLoop | https://github.com/Zoryvix/assureloop | competitor/open-source | category monitoring |

## Evidence hierarchy

1. primary regulation/spec
2. official project/vendor source for its own product
3. community firsthand experience
4. secondary commentary

Community complaints prove existence, not market size or willingness-to-pay.
Vendor claims are not independent market statistics.
