use eyre::Result;

static TEMPLATE: &str = r##"
<!DOCTYPE html>
<html lang="en">
  <body>
    <pre class="mermaid">
    __GRAPH__
    </pre>
    <script type="module">
      import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
    </script>
  </body>
</html>
"##;

pub fn render_mermaid(graph: &str) -> String {
    TEMPLATE.replace("__GRAPH__", graph)
}

pub fn test_graph() -> Result<()> {
    let html = render_mermaid(
        r##"
graph LR
    A --- B
    B-->C[fa:fa-ban forbidden]
    B-->D(fa:fa-spinner);
        "##,
    );

    let temp_file = tempfile::Builder::new().suffix(".html").tempfile()?;

    std::fs::write(temp_file.path(), html)?;

    open::that(temp_file.path())?;

    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}
