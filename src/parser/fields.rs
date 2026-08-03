/// Complete lookup table of all known incoPat searchable field codes.
/// Extracted from the official incoPat help documentation.
/// All comparisons should be case-insensitive.

const FIELD_CODES: &[&str] = &[
    // === Semantic search (also act as field codes) ===
    "R", "RAD", "RPD",

    // === Technical fields (技术字段) ===
    "TI", "TI-CN", "TI-OTLANG", "TI-EN", "TI-DWPI",
    "AB", "AB-CN", "AB-OTLANG", "AB-EN",
    "USE-DWPI", "ADV-DWPI", "NOVELTY-DWPI", "ABSTRACT-DWPI",
    "DTD-DWPI", "ACTIVITY-DWPI", "MEC-DWPI", "FOC-DWPI", "DRAW-DWPI",
    "TIAB", "TIAB-DWPI",
    "CLAIM", "FIRST-CLAIM", "FIRST-CLAIM-OR",
    "INDEPCLAIMS-CN", "DEPCLAIMS-CN",
    "NO-INDEPCLAIMS", "NO-DEPCLAIMS",
    "FIRST-CLAIM-TS", "LEN-FIRST-CLAIM",
    "CLAIM-EN", "CLAIM-CN", "CLAIM-OT", "NO-CLAIM",
    "TIABC",
    "DES", "DES-OT", "DES-EN", "DES-CN",
    "TECHNICAL-FIELD", "BACKGROUND-ART", "DISCLOSURE", "MODE-FOR-INVENTION",
    "NO-IMAGE",
    "EFFECT-S-CN", "USE-CN", "USE-EN",
    "EFFECT-PH-CN", "EFFECT-CN",
    "EFFECT-CN-3", "EFFECT-CN-2", "EFFECT-CN-1", "EFFECT-TRIZ",
    "ALL", "FULL",
    "FILING-LANG",
    "PRD-FLAG", "PRD", "PRD-DWPI",
    "PAGE",
    "VLSTAR", "VLSTAR-1", "VLSTAR-2", "VLSTAR-3",
    "REWARD-LEVEL", "REWARD-NAME", "REWARD-SESSION",
    "STD-TYPE", "STD-PROJECT", "STD-NUM", "STD-COMPANY", "STD-FLAG",
    "CAS-NO", "DRUG-NAME-CN", "DRUG-NAME-EN",
    "COMPANY", "BRAND-NAME", "ACTIVE-INGREDIENT", "TARGET", "INDICATION",
    "PATENT-EXPIRATION", "PED-PATENT-EXPIRATION",

    // === Company & people fields (公司&人字段) ===
    "WHO",
    "AP-ALL", "AP-GROUP", "AP-GROUPTT",
    "AP", "APTT", "AP-LITE", "AP-OR", "AP-OT", "AP-TS",
    "APNOR", "APNORTT", "AP-ROOT", "AP-FIRST",
    "AP-NEW-NAME", "AP-OTADD", "NO-AP",
    "PATENTEE", "PATENTEETT", "PATENTEENOR", "PATENTEENORTT",
    "ASSIGN-PARTY",
    "AOR", "AOR-TYPE",
    "AEE", "AEETT", "AEENOR", "AEENORTT", "AEE-TYPE",
    "IN", "INTT", "IN-OR", "IN-OT", "IN-TS",
    "IN-FIRST", "NO-IN", "IN-NEW-NAME", "IN-CURRENT",
    "LOR", "LEE", "LOR-TYPE", "LEE-TYPE",
    "OPPONENT",
    "AT", "AGC",
    "LGI-PARTY",
    "RE-AP", "IN-AP",
    "RI-ME", "RI-AE", "RI-LEADER",
    "POR", "PEE",
    "EX",
    "AP-TYPE", "PATENTEE-TYPE",
    "CO-DWPI", "CK-DWPI", "CK-TYPE-DWPI",
    "AP-AS", "AP-EN", "AP-REG-LOCATION",
    "AP-COMPANY-ORG-TYPE", "AP-ESTIBLISH-TIME",
    "AP-USC", "AP-REG-NUMBER", "AP-REG-STATUS", "AP-LIST-CODE",

    // === Classification fields (分类字段) ===
    "IPC", "IPC-MAIN",
    "IPC-SECTION", "IPC-CLASS", "IPC-SUBCLASS", "IPC-GROUP", "IPC-SUBGROUP",
    "IPCM-SECTION", "IPCM-CLASS", "IPCM-SUBCLASS", "IPCM-GROUP",
    "IPC-LOW", "IPC-HIGH",
    "IPCM-LOW", "IPCM-HIGH",
    "IPC-DWPI", "IPC-SECTION-DWPI", "IPC-CLASS-DWPI",
    "IPC-SUBCLASS-DWPI", "IPC-GROUP-DWPI", "IPC-SUBGROUP-DWPI",
    "IPC-F-DWPI", "IPC-SECTION-F-DWPI", "IPC-CLASS-F-DWPI",
    "IPC-SUBCLASS-F-DWPI", "IPC-GROUP-F-DWPI", "IPC-SUBGROUP-F-DWPI",
    "DC-DWPI", "DC-SECTION-DWPI", "DC-CLASS-DWPI",
    "MC-DWPI", "MC-SECTION-DWPI", "MC-CLASS-DWPI",
    "MC-GROUP-DWPI", "MC-SUBGROUP-DWPI", "MC-SUBGROUPD-DWPI",
    "MC-FULLMC-DWPI", "MC-FULLMCX-DWPI",
    "LOC", "LOC-CLASS", "LOC-SUBCLASS",
    "ECLA", "ECLA-SECTION", "ECLA-CLASS", "ECLA-SUBCLASS",
    "ECLA-GROUP", "ECLA-SUBGROUP",
    "UC", "UC-MAIN",
    "CPC", "CPC-SECTION", "CPC-CLASS", "CPC-SUBCLASS",
    "CPC-GROUP", "CPC-SUBGROUP",
    "CPC-MAIN", "CPCM-SECTION", "CPCM-CLASS", "CPCM-SUBCLASS",
    "CPCM-GROUP", "CPCM-SUBGROUP",
    "FI", "FT",
    "CLASS",
    "BCLASS", "MBCLAS1", "MBCLAS2", "MBCLAS3", "MBCLAS4", "MBCLASS",
    "BCLAS1", "BCLAS2", "BCLAS3", "BCLAS4",
    "INDUSTRY1", "MINDUSTRY1", "MINDUSTRY2", "INDUSTRY2", "INDUSTRY-TYPE",
    "MKCLAS1", "MKCLAS2",
    "SC-MAIN", "SC-SECTION", "SC-CLASS", "SC-SUBCLASS",
    "LNGCLAS1", "LNGCLAS2", "LNGCLAS3",
    "CPCLAS1", "CPCLAS2", "CPCLAS3",
    "DIGCLAS1", "DIGCLAS2", "DIGCLAS3",

    // === Region fields (地域字段) ===
    "AP-COUNTRY", "IN-COUNTRY",
    "AUTH", "PNC",
    "AP-ADD", "PR-AU", "PR-AU-DWPI", "ORIPRC-DWPI",
    "AP-PROVINCE", "PC-CN", "AP-PC",
    "CITY", "COUNTY",
    "PATENTEE-ADD", "PATENTEE-PROVINCE", "PATENTEE-CITY", "PATENTEE-COUNTY",
    "IN-ADD", "IN-ADD-OTH", "IN-OR-ADD",
    "IN-CITY", "IN-STATE",
    "ASSIGN-COUNTRY", "ASSIGNEE-ADD", "ASSIGNEE-CADD",
    "ASSIGN-STATE", "ASSIGN-CITY",
    "AEE-PROVINCE", "AEE-CITY", "AEE-COUNTY",
    "ASSIGNOR-ADD", "AOR-PROVINCE", "AOR-CITY", "AOR-COUNTY",
    "AT-COUNTRY", "AT-ADD", "AT-CITY", "AT-STATE",
    "LGI-REGION",
    "WHERE",
    "DE-COUNTRY",

    // === Number fields (号码字段) ===
    "AN", "ANN",
    "PN", "PNN",
    "PU-PN", "GRANT-PN",
    "RPND-DWPI",
    "PR", "PR-DWPI", "PRN",
    "PT", "PAT", "PNK",
    "MF", "CF", "MFN", "CFN",
    "IF", "IFN",
    "F-DWPI", "FN-DWPI",
    "FA-COUNTRY", "FA-COUNTRY-DWPI", "FCN-DWPI",
    "NUMBER",
    "RI-NUM", "RI-INERNAL",
    "LICENSE-NO", "PLEDGE-NO",
    "IAN", "IPN",
    "SAN", "SUBSAN", "ESM",
    "CONTINUATION-PARENT", "CONTINUATION-INPART-PARENT",

    // === Date fields (日期字段) ===
    "AD", "RADD-DWPI", "ADM", "ADY",
    "PD", "PU-DATE", "PU-YEAR", "PU-MONTH",
    "PDY", "PDM",
    "PR-DATE", "PR-DATE-DWPI", "PRYEAR",
    "ORI-PRDATE", "ORI-PRYEAR", "ORI-PRYEAR-DWPI",
    "CT-AD", "CT-PD", "CTFW-AD", "CTFW-PD", "CTYEAR",
    "SUBEX-DATE",
    "GRANT-DATE", "GRANT-YEAR", "GRANT-MONTH",
    "EXDT", "EXDT-YEAR", "EXDT-MONTH",
    "EXPIRY-DATE", "EXPIRY-YEAR",
    "ECD",
    "PLEDGEYEAR", "ASSIGNYEAR", "LICENSEYEAR",
    "ASSIGN-DATE", "ASSIGN-RD",
    "RI-DATE", "LGI-DATE", "LGI-FD", "LGI-CD", "LGD",
    "PLEDGE-DATE", "LICENSE-DATE",
    "LICENSE-SD", "LICENSE-TD",
    "PLEDGE-CD", "PLEDGE-RD",
    "LGIYEAR", "LGI-FY", "LGI-CY",
    "PATENT-LIFE", "EX-TIME", "PFEX-TIME",
    "RE-DATE", "IN-DATE", "OR-DATE",
    "REAPP-DATE", "INAPP-DATE",

    // === Legal fields (法律字段) ===
    "STATUS", "STATUS-LITE",
    "LG", "LGE", "LGF", "LGC",
    "RI-TYPE", "RI-TEXT", "RI-AP",
    "RE-DECISION", "RI-BASIS", "RI-POINT",
    "LGI-COURT", "LGI-JUDGE", "LGI-FIRM", "LAWYER", "LGI-CAUSE",
    "ASSIGN-TEXT",
    "LGI-TI", "LGI-TEXT", "LGI-TYPE", "LGI-NO",
    "LGI-PROCEDURE", "LGI-PLAINTIFF", "LGI-DEFENDANT",
    "LICENSE-TYPE", "LICENSE-STAGE", "LICENSE-CS",
    "LEE-CURRENT", "PEE-CURRENT",
    "PLEDGE-TYPE", "PLEDGE-STAGE",
    "LAWTXT",
    "ASSIGN-FLAG", "ASSIGN-TIMES", "ASSIGN-NO", "ASSIGN-TYPE",
    "LICENCE-FLAG", "LICENCE-TIMES",
    "PLEGE-FLAG", "PLEDGE-TIMES",
    "REE-FLAG",
    "LGI-FLAG", "LGI-TIMES",
    "ACTION-TYPES",
    "CUSTOMS-FLAG", "ALL-FLAG",
    "TOVALIDE-DATE",
    "FLAG-337",

    // === Citation fields (引证字段) ===
    "CT", "CTFW",
    "CT-SELF", "CT-OTH", "CTFW-SELF", "CTFW-OTH",
    "CT-TIMES", "CTFW-TIMES",
    "CT-SELF-TIMES", "CT-OTH-TIMES",
    "CTFW-SELF-TIMES", "CTFW-OTH-TIMES",
    "FCT", "FCTFW",
    "CT-AP", "CTFW-AP", "FCT-AP", "FCTFW-AP",
    "CT-NO", "CTFW-NO",
    "CT-AUTH", "CTFW-AUTH",
    "CT-CODE", "CT-X",
    "FCT-TIMES", "FCTFW-TIMES",
    "CTNP",
    "CT-SOURCE", "CTFW-SOURCE",

    // === Other/miscellaneous fields ===
    "DOC-DC",
    "RAND-DWPI",
];

/// Semantic search keywords that also function as field codes.
const SEMANTIC_KEYWORDS: &[&str] = &["R", "RAD", "RPD"];

/// Check if a given identifier is a known incoPat field code (case-insensitive).
pub fn is_field_code(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    FIELD_CODES.iter().any(|&code| code == upper)
}

/// Check if a given identifier is a semantic search keyword (case-insensitive).
pub fn is_semantic_keyword(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    SEMANTIC_KEYWORDS.iter().any(|&kw| kw == upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_technical_fields() {
        assert!(is_field_code("ti"));
        assert!(is_field_code("TI"));
        assert!(is_field_code("Ti"));
        assert!(is_field_code("ab"));
        assert!(is_field_code("tiab"));
        assert!(is_field_code("claim"));
        assert!(is_field_code("des"));
        assert!(is_field_code("full"));
        assert!(is_field_code("all"));
    }

    #[test]
    fn known_hyphenated_fields() {
        assert!(is_field_code("ti-cn"));
        assert!(is_field_code("TI-EN"));
        assert!(is_field_code("ti-dwpi"));
        assert!(is_field_code("effect-cn-3"));
        assert!(is_field_code("ap-group"));
        assert!(is_field_code("first-claim"));
    }

    #[test]
    fn known_company_fields() {
        assert!(is_field_code("ap"));
        assert!(is_field_code("aee"));
        assert!(is_field_code("in"));
        assert!(is_field_code("lor"));
        assert!(is_field_code("lee"));
    }

    #[test]
    fn known_classification_fields() {
        assert!(is_field_code("ipc"));
        assert!(is_field_code("cpc"));
        assert!(is_field_code("loc"));
        assert!(is_field_code("uc"));
        assert!(is_field_code("fi"));
    }

    #[test]
    fn known_date_fields() {
        assert!(is_field_code("ad"));
        assert!(is_field_code("pd"));
        assert!(is_field_code("pr-date"));
        assert!(is_field_code("GRANT-DATE"));
        assert!(is_field_code("EXDT"));
    }

    #[test]
    fn semantic_keywords() {
        assert!(is_semantic_keyword("R"));
        assert!(is_semantic_keyword("r"));
        assert!(is_semantic_keyword("RAD"));
        assert!(is_semantic_keyword("rad"));
        assert!(is_semantic_keyword("RPD"));
        assert!(is_semantic_keyword("rpd"));
        assert!(!is_semantic_keyword("AND"));
        assert!(!is_semantic_keyword("ti"));
    }

    #[test]
    fn unknown_fields() {
        assert!(!is_field_code("UNKNOWN"));
        assert!(!is_field_code("hello"));
        assert!(!is_field_code("AND"));
        assert!(!is_field_code("OR"));
    }
}
