use eyre::Result;

static TEMPLATE: &str = r##"
<!DOCTYPE html>
<html lang="en">
  <body>
    <script>
      function zoomIn() { changeZoom(1.2); }
      function zoomOut() { changeZoom(0.8); }
      function changeZoom(multiplier) {
        const svg = document.querySelector('svg');
        const currentWidth = parseFloat(svg.getAttribute('width').replace('%', ''));
        svg.setAttribute('width', (currentWidth * multiplier) + '%');
      }
    </script>
    <button onclick="zoomIn()">Zoom +</button>
    <button onclick="zoomOut()">Zoom -</button>

    <pre class="mermaid">
    __GRAPH__
    </pre>
    <script type="module">
      import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
    </script>
  </body>
</html>
"##;

pub struct Node {
    id: String,
    label: String,
}

impl Node {
    pub fn id(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            label: id,
        }
    }

    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"{}["`{}`"]"#, self.id, self.label)
    }
}

pub struct Edge {
    from: String,
    to: String,
    label: Option<String>,
}

impl Edge {
    pub fn from_to(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
        }
    }

    pub fn new(from: String, to: String, label: Option<String>) -> Self {
        Self { from, to, label }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.label {
            Some(label) => write!(f, "{} -->|{}| {}", self.from, label, self.to),
            None => write!(f, "{} --> {}", self.from, self.to),
        }
    }
}

pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        Self { nodes, edges }
    }

    pub fn render(&self) -> String {
        let mut src = "graph TB\n".to_string();
        for node in &self.nodes {
            src.push_str(&format!("    {}\n", node));
        }
        for edge in &self.edges {
            src.push_str(&format!("    {}\n", edge));
        }
        src
    }

    pub fn render_and_open(&self) -> Result<()> {
        let html = render_mermaid(&self.render());
        let temp_file = tempfile::Builder::new().suffix(".html").tempfile()?;

        std::fs::write(temp_file.path(), html)?;
        open::that(temp_file.path())?;
        // std::thread::sleep(std::time::Duration::from_secs(3));
        let _ = temp_file.keep()?;
        Ok(())
    }
}

pub fn render_mermaid(graph: &str) -> String {
    TEMPLATE.replace("__GRAPH__", graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let nodes = vec![
            Node::new("A", "Node A"),
            Node::new("B", "Node B"),
            Node::new("C", "Node C"),
        ];

        let edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                label: None,
            },
            Edge {
                from: "B".into(),
                to: "C".into(),
                label: Some("Edge Label".into()),
            },
        ];

        let g = Graph::new(nodes, edges);

        let html = render_mermaid(&g.render());

        assert_eq!(
            html,
            r##"
fooo
"##
        );
    }
}
