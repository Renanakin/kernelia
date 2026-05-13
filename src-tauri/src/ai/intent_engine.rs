use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    Network,
    Performance,
    Security,
    Maintenance,
    Drivers,
    Files,
    Services,
    Updates,
    GeneralSupport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmotionCategory {
    Neutral,
    Frustrated,
    Urgent,
    Confused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CriticalityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyLevel {
    Low,
    Normal,
    High,
    Immediate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub label: String,
    pub confidence: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    pub normalized_text: String,
    pub intent: IntentCategory,
    pub emotion: EmotionCategory,
    pub criticality: CriticalityLevel,
    pub urgency: UrgencyLevel,
    pub symptoms: Vec<String>,
    pub operational_tags: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub recommended_plan: Vec<String>,
    pub confidence: f32,
}

pub fn analyze_message(user_message: &str, recent_context: &[String]) -> IntentAnalysis {
    let normalized = normalize_text(user_message);
    let all_text = build_context_text(&normalized, recent_context);

    let intent = classify_intent(&all_text);
    let emotion = classify_emotion(&normalized);
    let urgency = detect_urgency(&all_text);
    let symptoms = detect_symptoms(&all_text);
    let operational_tags = build_operational_tags(&intent, &urgency, &symptoms);
    let hypotheses = build_hypotheses(&intent, &symptoms, &all_text);
    let criticality = classify_criticality(&intent, &urgency, &symptoms, &all_text);
    let recommended_plan = decision_plan(&intent, &criticality, &symptoms);
    let confidence = estimate_confidence(&intent, &symptoms, &hypotheses);

    IntentAnalysis {
        normalized_text: normalized,
        intent,
        emotion,
        criticality,
        urgency,
        symptoms,
        operational_tags,
        hypotheses,
        recommended_plan,
        confidence,
    }
}

pub fn to_operational_context(a: &IntentAnalysis) -> String {
    let hypotheses = a
        .hypotheses
        .iter()
        .map(|h| format!("{} ({:.0}%)", h.label, h.confidence * 100.0))
        .collect::<Vec<_>>()
        .join(", ");
    let steps = a.recommended_plan.join(" | ");

    format!(
        "[KERNEL_INTENT]\nintent={:?}\nemotion={:?}\ncriticality={:?}\nurgency={:?}\nsymptoms={}\ntags={}\nhypotheses={}\nplan={}",
        a.intent,
        a.emotion,
        a.criticality,
        a.urgency,
        a.symptoms.join(", "),
        a.operational_tags.join(", "),
        hypotheses,
        steps
    )
}

fn normalize_text(input: &str) -> String {
    let lowered = input.to_lowercase();
    lowered
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace('ñ', "n")
        .replace("  ", " ")
        .trim()
        .to_string()
}

fn build_context_text(normalized: &str, recent_context: &[String]) -> String {
    let mut all = String::new();
    all.push_str(normalized);
    for item in recent_context {
        all.push(' ');
        all.push_str(&normalize_text(item));
    }
    all
}

fn classify_intent(text: &str) -> IntentCategory {
    if contains_any(
        text,
        &[
            "dns", "wifi", "internet", "red", "gateway", "latencia", "ping",
        ],
    ) {
        return IntentCategory::Network;
    }
    if contains_any(
        text,
        &[
            "lento",
            "rendimiento",
            "cpu",
            "ram",
            "temperatura",
            "congelado",
        ],
    ) {
        return IntentCategory::Performance;
    }
    if contains_any(
        text,
        &[
            "virus",
            "malware",
            "defender",
            "firewall",
            "seguridad",
            "intruso",
        ],
    ) {
        return IntentCategory::Security;
    }
    if contains_any(
        text,
        &["driver", "controlador", "audio", "gpu", "usb", "pantalla"],
    ) {
        return IntentCategory::Drivers;
    }
    if contains_any(
        text,
        &["archivo", "escritorio", "carpeta", "documento", "explorar"],
    ) {
        return IntentCategory::Files;
    }
    if contains_any(
        text,
        &["servicio", "service", "spooler", "reiniciar servicio"],
    ) {
        return IntentCategory::Services;
    }
    if contains_any(
        text,
        &["update", "actualizacion", "windows update", "parche"],
    ) {
        return IntentCategory::Updates;
    }
    if contains_any(
        text,
        &[
            "limpieza",
            "temporales",
            "mantenimiento",
            "optimizar",
            "cache",
        ],
    ) {
        return IntentCategory::Maintenance;
    }
    IntentCategory::GeneralSupport
}

fn classify_emotion(text: &str) -> EmotionCategory {
    if contains_any(text, &["urgente", "ahora", "ya", "inmediato"]) {
        return EmotionCategory::Urgent;
    }
    if contains_any(
        text,
        &[
            "no funciona",
            "fallo",
            "error",
            "se cayo",
            "crash",
            "colgado",
        ],
    ) {
        return EmotionCategory::Frustrated;
    }
    if contains_any(text, &["no entiendo", "que hago", "ayuda", "como"]) {
        return EmotionCategory::Confused;
    }
    EmotionCategory::Neutral
}

fn detect_urgency(text: &str) -> UrgencyLevel {
    if contains_any(
        text,
        &[
            "caido",
            "produccion",
            "critico",
            "no arranca",
            "no puedo trabajar",
            "inmediato",
        ],
    ) {
        return UrgencyLevel::Immediate;
    }
    if contains_any(
        text,
        &["urgente", "hoy", "bloqueado", "sin internet", "no responde"],
    ) {
        return UrgencyLevel::High;
    }
    if contains_any(text, &["cuando puedas", "revisar", "mejorar"]) {
        return UrgencyLevel::Low;
    }
    UrgencyLevel::Normal
}

fn detect_symptoms(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    push_if(
        text,
        &mut out,
        "sin internet",
        &["sin internet", "no hay internet", "internet malo"],
    );
    push_if(
        text,
        &mut out,
        "latencia alta",
        &["latencia", "ping alto", "lento internet"],
    );
    push_if(
        text,
        &mut out,
        "alto uso cpu",
        &["cpu alta", "cpu al", "procesador al 100"],
    );
    push_if(
        text,
        &mut out,
        "alto uso ram",
        &["ram llena", "sin memoria", "ram alta"],
    );
    push_if(
        text,
        &mut out,
        "errores drivers",
        &["driver", "controlador", "dispositivo desconocido"],
    );
    push_if(
        text,
        &mut out,
        "servicio detenido",
        &["servicio detenido", "servicio caido", "service stopped"],
    );
    push_if(
        text,
        &mut out,
        "fallo actualizaciones",
        &["windows update", "no actualiza", "error update"],
    );
    push_if(
        text,
        &mut out,
        "riesgo seguridad",
        &["malware", "virus", "amenaza", "intruso"],
    );
    out
}

fn build_operational_tags(
    intent: &IntentCategory,
    urgency: &UrgencyLevel,
    symptoms: &[String],
) -> Vec<String> {
    let mut tags = vec![format!("intent:{:?}", intent).to_lowercase()];
    tags.push(format!("urgency:{:?}", urgency).to_lowercase());
    for symptom in symptoms {
        tags.push(format!("symptom:{}", symptom.replace(' ', "_")));
    }
    tags
}

fn build_hypotheses(intent: &IntentCategory, symptoms: &[String], text: &str) -> Vec<Hypothesis> {
    let mut items = Vec::new();

    match intent {
        IntentCategory::Network => {
            items.push(Hypothesis {
                label: "Problema DNS o gateway".to_string(),
                confidence: if symptoms.iter().any(|s| s == "sin internet") {
                    0.82
                } else {
                    0.64
                },
                rationale: "Sintomas de conectividad apuntan a DNS/gateway".to_string(),
            });
            if contains_any(text, &["wifi"]) {
                items.push(Hypothesis {
                    label: "Inestabilidad WiFi".to_string(),
                    confidence: 0.58,
                    rationale: "Solicitud menciona capa inalambrica".to_string(),
                });
            }
        }
        IntentCategory::Drivers => {
            items.push(Hypothesis {
                label: "Controlador faltante o corrupto".to_string(),
                confidence: 0.8,
                rationale: "Mencion directa de drivers/controladores".to_string(),
            });
        }
        IntentCategory::Performance => {
            items.push(Hypothesis {
                label: "Saturacion de recursos".to_string(),
                confidence: 0.76,
                rationale: "Indicadores de lentitud y consumo".to_string(),
            });
        }
        IntentCategory::Security => {
            items.push(Hypothesis {
                label: "Incidente de seguridad activo".to_string(),
                confidence: 0.74,
                rationale: "Terminos de amenaza o malware presentes".to_string(),
            });
        }
        _ => {
            items.push(Hypothesis {
                label: "Se requiere diagnostico basico guiado".to_string(),
                confidence: 0.55,
                rationale: "Consulta general con baja especificidad".to_string(),
            });
        }
    }

    items.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
}

fn classify_criticality(
    intent: &IntentCategory,
    urgency: &UrgencyLevel,
    symptoms: &[String],
    text: &str,
) -> CriticalityLevel {
    if matches!(urgency, UrgencyLevel::Immediate)
        || contains_any(text, &["produccion", "empresa detenida", "no arranca"])
    {
        return CriticalityLevel::Critical;
    }

    if matches!(urgency, UrgencyLevel::High)
        || symptoms
            .iter()
            .any(|s| s == "riesgo seguridad" || s == "sin internet")
    {
        return CriticalityLevel::High;
    }

    if matches!(
        intent,
        IntentCategory::Maintenance | IntentCategory::GeneralSupport
    ) && symptoms.is_empty()
    {
        return CriticalityLevel::Low;
    }

    CriticalityLevel::Medium
}

fn decision_plan(
    intent: &IntentCategory,
    criticality: &CriticalityLevel,
    symptoms: &[String],
) -> Vec<String> {
    let mut plan = Vec::new();

    plan.push("Validar contexto operativo y permisos de ejecucion".to_string());

    match intent {
        IntentCategory::Network => {
            plan.push("Ejecutar ping/traceroute y validacion DNS".to_string());
            plan.push("Correlacionar gateway, DHCP y perdida de paquetes".to_string());
        }
        IntentCategory::Drivers => {
            plan.push("Listar drivers con error y codigos de dispositivo".to_string());
            plan.push("Sugerir actualizacion o reinstalacion por fabricante".to_string());
        }
        IntentCategory::Performance => {
            plan.push("Medir CPU/RAM/disco y procesos top".to_string());
            plan.push("Aplicar acciones de mantenimiento seguras".to_string());
        }
        IntentCategory::Security => {
            plan.push("Ejecutar chequeo Defender y puertos activos".to_string());
            plan.push("Aislar procesos sospechosos y registrar evidencia".to_string());
        }
        _ => {
            plan.push("Ejecutar diagnostico base del sistema".to_string());
        }
    }

    if matches!(
        criticality,
        CriticalityLevel::High | CriticalityLevel::Critical
    ) {
        plan.push("Priorizar respuesta inmediata y reporte de riesgo".to_string());
    }

    if symptoms.is_empty() {
        plan.push("Solicitar un sintoma concreto para elevar precision".to_string());
    }

    plan
}

fn estimate_confidence(
    intent: &IntentCategory,
    symptoms: &[String],
    hypotheses: &[Hypothesis],
) -> f32 {
    let base = match intent {
        IntentCategory::GeneralSupport => 0.55,
        _ => 0.68,
    };

    let symptom_boost = (symptoms.len() as f32 * 0.05).min(0.2);
    let hypothesis_boost = hypotheses
        .first()
        .map(|h| (h.confidence - 0.5).max(0.0) * 0.3)
        .unwrap_or(0.0);

    (base + symptom_boost + hypothesis_boost).clamp(0.0, 0.98)
}

fn contains_any(text: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| text.contains(term))
}

fn push_if(text: &str, out: &mut Vec<String>, label: &str, terms: &[&str]) {
    if contains_any(text, terms) {
        out.push(label.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_message, CriticalityLevel, IntentCategory};

    #[test]
    fn detects_network_intent() {
        let a = analyze_message("el internet esta malo y tengo ping alto", &[]);
        assert_eq!(a.intent, IntentCategory::Network);
    }

    #[test]
    fn elevates_criticality_when_down() {
        let a = analyze_message("produccion caida, no arranca y es urgente", &[]);
        assert_eq!(a.criticality, CriticalityLevel::Critical);
    }
}
