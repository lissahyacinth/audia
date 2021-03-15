struct StringConfidence {
    confidence: f32,
    text: String,
}

struct Prediction {
    timestamp: usize,
    duration: usize,
    terms: Vec<StringConfidence>,
}
