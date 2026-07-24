use crate::rag::models::{DomainSpecialty, QueryAnalysis, QueryCategory};

pub fn analyze_query(message: &str) -> QueryAnalysis {
    let normalized_text = normalize_text(message);
    let tokens = tokenize(&normalized_text);
    let specialty = detect_specialty(&normalized_text);
    let symptoms = detect_symptoms(&normalized_text);
    let entities = detect_entities(&normalized_text);
    let query_category = classify_query_category(&normalized_text, &tokens, &symptoms, specialty.clone());
    let ambiguity_score =
        compute_ambiguity_score(&normalized_text, &tokens, &symptoms, &entities, &specialty);
    let urgency = detect_urgency(message, &tokens);
    let requires_clarification = ambiguity_score >= 0.68
        || matches!(query_category, QueryCategory::Short | QueryCategory::Ambiguous);

    QueryAnalysis {
        normalized_text,
        query_category,
        specialty,
        urgency,
        symptoms,
        entities,
        ambiguity_score,
        requires_clarification,
    }
}

fn normalize_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());

    for ch in input.chars() {
        let mapped = match ch {
            'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => 'a',
            'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
            'ñ' | 'Ñ' => 'n',
            _ => ch.to_ascii_lowercase(),
        };

        if mapped.is_ascii_alphanumeric() || mapped.is_ascii_whitespace() {
            normalized.push(mapped);
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize(normalized_text: &str) -> Vec<&str> {
    normalized_text.split_whitespace().collect()
}

fn detect_specialty(text: &str) -> DomainSpecialty {
    let mut best_specialty = DomainSpecialty::Unknown;
    let mut best_score = 0_i32;

    for (specialty, keywords) in specialty_rules() {
        let score = keywords
            .iter()
            .filter(|keyword| text.contains(**keyword))
            .count() as i32;

        if score > best_score {
            best_specialty = specialty.clone();
            best_score = score;
        }
    }

    best_specialty
}

fn specialty_rules() -> Vec<(DomainSpecialty, &'static [&'static str])> {
    vec![
        (
            DomainSpecialty::Network,
            &[
                "dns",
                "ipconfig",
                "ping",
                "gateway",
                "ethernet",
                "wifi",
                "internet",
                "adaptador de red",
                "latencia",
                "red",
                "nslookup",
            ],
        ),
        (
            DomainSpecialty::Services,
            &[
                "spooler",
                "servicio",
                "services",
                "service",
                "print spooler",
                "detenido",
                "no imprime",
            ],
        ),
        (
            DomainSpecialty::Drivers,
            &[
                "driver",
                "controlador",
                "codigo 43",
                "code 43",
                "gpu",
                "audio",
                "usb",
                "pantalla",
                "dispositivo",
            ],
        ),
        (
            DomainSpecialty::Performance,
            &[
                "disco al 100",
                "100 de disco",
                "cpu al 100",
                "ram alta",
                "lento",
                "rendimiento",
                "congelado",
                "latencia del sistema",
                "alto consumo",
            ],
        ),
        (
            DomainSpecialty::Security,
            &[
                "defender",
                "firewall",
                "malware",
                "virus",
                "intruso",
                "seguridad",
                "amenaza",
                "ransomware",
            ],
        ),
        (
            DomainSpecialty::Maintenance,
            &[
                "sfc",
                "dism",
                "mantenimiento",
                "reparacion",
                "integridad",
                "temporales",
                "limpieza",
            ],
        ),
        (
            DomainSpecialty::Filesystem,
            &[
                "archivo",
                "carpeta",
                "escritorio",
                "disco c",
                "permisos",
                "ruta",
                "explorador",
                "filesystem",
            ],
        ),
        (
            DomainSpecialty::Software,
            &[
                "instalar",
                "desinstalar",
                "programa",
                "aplicacion",
                "actualizacion",
                "actualizaciones",
                "windows feature",
                "feature",
                "winget",
                "update",
                "software",
            ],
        ),
        (
            DomainSpecialty::Processes,
            &[
                "proceso",
                "tasklist",
                "task manager",
                "consumo de cpu",
                "proceso colgado",
                "process",
            ],
        ),
        (
            DomainSpecialty::System,
            &[
                "windows",
                "arranque",
                "inicio",
                "kernel",
                "sistema",
                "equipo",
            ],
        ),
        (
            DomainSpecialty::SensitiveOps,
            &[
                "formatea",
                "borrar system32",
                "desactivar defender",
                "deshabilitar firewall",
                "regedit",
                "registro",
            ],
        ),
    ]
}

fn detect_symptoms(text: &str) -> Vec<String> {
    let symptom_rules = [
        ("dns_failure", &["dns", "nslookup", "no resuelve", "resolucion"] as &[_]),
        ("network_down", &["sin internet", "no hay internet", "sin red", "wifi caido"]),
        ("high_latency", &["latencia", "ping alto", "ping lento"]),
        ("print_failure", &["no imprime", "spooler", "impresora", "cola de impresion"]),
        ("driver_code_43", &["codigo 43", "code 43"]),
        ("disk_100", &["disco al 100", "100 de disco", "disco saturado"]),
        ("high_cpu", &["cpu al 100", "alto consumo de cpu"]),
        ("slow_system", &["muy lento", "equipo lento", "se congela", "congelado"]),
        ("update_failure", &["windows update", "no actualiza", "fallo actualizacion"]),
        ("security_risk", &["malware", "virus", "intruso", "amenaza"]),
    ];

    symptom_rules
        .iter()
        .filter(|(_, aliases)| aliases.iter().any(|alias| text.contains(alias)))
        .map(|(label, _)| (*label).to_string())
        .collect()
}

fn detect_entities(text: &str) -> Vec<String> {
    let entity_rules = [
        ("dns", &["dns", "nslookup"] as &[_]),
        ("spooler", &["spooler", "print spooler", "cola de impresion"]),
        ("gpu", &["gpu", "tarjeta grafica", "video"]),
        ("disk", &["disco", "ssd", "hdd"]),
        ("desktop", &["escritorio", "desktop"]),
        ("file_inventory", &["archivo", "archivos"]),
        ("firewall", &["firewall"]),
        ("defender", &["defender"]),
        ("windows_update", &["windows update", "actualizacion", "actualizaciones", "update", "winget"]),
        ("sfc", &["sfc"]),
        ("dism", &["dism"]),
    ];

    entity_rules
        .iter()
        .filter(|(_, aliases)| aliases.iter().any(|alias| text.contains(alias)))
        .map(|(entity, _)| (*entity).to_string())
        .collect()
}

fn classify_query_category(
    text: &str,
    tokens: &[&str],
    symptoms: &[String],
    specialty: DomainSpecialty,
) -> QueryCategory {
    if contains_any(
        text,
        &[
            "formatea",
            "borra",
            "elimina",
            "desactiva defender",
            "deshabilita firewall",
            "mata el proceso",
            "borra system32",
        ],
    ) {
        return QueryCategory::UnsafeRequest;
    }

    if tokens.len() <= 1 {
        return QueryCategory::Short;
    }

    if specialty == DomainSpecialty::Unknown && symptoms.is_empty() {
        return QueryCategory::OutOfDomain;
    }

    if symptoms.is_empty() && contains_any(text, &["no funciona", "falla", "error", "problema"]) {
        return QueryCategory::Ambiguous;
    }

    if contains_any(
        text,
        &[
            "revisa",
            "diagnostica",
            "ejecuta",
            "reinicia",
            "repara",
            "arregla",
            "soluciona",
        ],
    ) {
        return QueryCategory::ActionRequest;
    }

    if !symptoms.is_empty() {
        return QueryCategory::SymptomBased;
    }

    QueryCategory::Specific
}

fn compute_ambiguity_score(
    text: &str,
    tokens: &[&str],
    symptoms: &[String],
    entities: &[String],
    specialty: &DomainSpecialty,
) -> f32 {
    let mut score = 0.15_f32;

    if tokens.len() <= 2 {
        score += 0.35;
    }
    if specialty == &DomainSpecialty::Unknown {
        score += 0.25;
    }
    if symptoms.is_empty() {
        score += 0.15;
    }
    if entities.is_empty() {
        score += 0.10;
    }
    if contains_any(text, &["no funciona", "ayuda", "error", "problema"]) {
        score += 0.20;
    }

    score.clamp(0.0, 1.0)
}

fn detect_urgency(original_text: &str, tokens: &[&str]) -> String {
    let lowered = original_text.to_lowercase();

    if contains_any(
        &lowered,
        &["critico", "crítico", "urgente", "caido", "caído", "produccion", "producción"],
    ) {
        return "high".to_string();
    }

    if tokens.iter().any(|token| *token == "ahora" || *token == "ya") {
        return "high".to_string();
    }

    "normal".to_string()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_dns_queries_to_network() {
        let analysis = analyze_query("No tengo internet, revisa DNS por favor");
        assert_eq!(analysis.specialty, DomainSpecialty::Network);
        assert_eq!(analysis.query_category, QueryCategory::ActionRequest);
        assert!(analysis.symptoms.iter().any(|s| s == "dns_failure"));
        assert!(!analysis.requires_clarification);
    }

    #[test]
    fn routes_spooler_queries_to_services() {
        let analysis = analyze_query("Reinicia el spooler porque la impresora no imprime");
        assert_eq!(analysis.specialty, DomainSpecialty::Services);
        assert_eq!(analysis.query_category, QueryCategory::ActionRequest);
        assert!(analysis.entities.iter().any(|e| e == "spooler"));
    }

    #[test]
    fn routes_code_43_queries_to_drivers() {
        let analysis = analyze_query("Mi GPU muestra codigo 43 en Windows");
        assert_eq!(analysis.specialty, DomainSpecialty::Drivers);
        assert_eq!(analysis.query_category, QueryCategory::SymptomBased);
        assert!(analysis.symptoms.iter().any(|s| s == "driver_code_43"));
    }

    #[test]
    fn routes_disk_100_queries_to_performance() {
        let analysis = analyze_query("El equipo esta muy lento, tengo el disco al 100");
        assert_eq!(analysis.specialty, DomainSpecialty::Performance);
        assert_eq!(analysis.query_category, QueryCategory::SymptomBased);
        assert!(analysis.symptoms.iter().any(|s| s == "disk_100"));
    }

    #[test]
    fn routes_desktop_file_inventory_to_filesystem() {
        let analysis = analyze_query("Cuantos archivos tengo en el escritorio");
        assert_eq!(analysis.specialty, DomainSpecialty::Filesystem);
        assert_eq!(analysis.query_category, QueryCategory::Specific);
        assert!(analysis.entities.iter().any(|e| e == "desktop"));
    }

    #[test]
    fn routes_app_update_questions_to_software() {
        let analysis = analyze_query("Revisa las actualizaciones de aplicaciones");
        assert_eq!(analysis.specialty, DomainSpecialty::Software);
        assert_eq!(analysis.query_category, QueryCategory::ActionRequest);
        assert!(analysis.entities.iter().any(|e| e == "windows_update"));
    }

    #[test]
    fn marks_ambiguous_queries_for_clarification() {
        let analysis = analyze_query("ayuda");
        assert_eq!(analysis.query_category, QueryCategory::Short);
        assert!(analysis.requires_clarification);
        assert!(analysis.ambiguity_score >= 0.68);
    }
}
