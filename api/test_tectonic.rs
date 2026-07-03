fn main() {
    let tex = r#"\documentclass{article}\begin{document}Hello\end{document}"#;
    let _pdf = tectonic::latex_to_pdf(tex).unwrap();
}
