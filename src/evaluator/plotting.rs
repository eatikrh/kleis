use crate::ast::Expression;

use super::Evaluator;

impl Evaluator {
    /// diagram(options, element1, element2, ...) - Compose plot elements and render to SVG
    pub(crate) fn builtin_diagram(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::{compile_diagram, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("diagram() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // v0.96: Named arguments produce a trailing record
        // Check both first arg (legacy) and last arg (v0.96 style)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        // Check first arg for options record (legacy style)
        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        // Check last arg for options record (v0.96 named arguments style)
        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        // Collect plot elements from middle args
        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            // Handle lists of PlotElements (for dynamic generation with list_map)
            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation {
                name,
                args: _elem_args,
                ..
            } = &evaluated
            {
                if name == "PlotElement" {
                    // Decode PlotElement from expression
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    // Skip stray records (already processed)
                    continue;
                } else {
                    return Err(format!(
                        "diagram() expects PlotElement, got: {}(). Use plot(), bar(), scatter(), etc.",
                        name
            ));
                }
            } else {
                return Err(format!(
                    "diagram() expects PlotElement, got: {:?}",
                    evaluated
                ));
            }
        }

        if elements.is_empty() {
            return Err("diagram() requires at least one plot element".to_string());
        }

        // Compile to SVG and print with PLOT_SVG prefix for Jupyter kernel to detect
        match compile_diagram(&elements, &options) {
            Ok(output) => {
                // Print SVG with marker for Jupyter kernel
                println!("PLOT_SVG:{}", output.svg);
                Ok(Some(Expression::operation(
                    "PlotSVG",
                    vec![
                        Expression::Const(format!("{:.0}", output.width)),
                        Expression::Const(format!("{:.0}", output.height)),
                    ],
                )))
            }
            Err(e) => Err(format!("diagram() failed: {}", e)),
        }
    }

    /// Export a diagram as Typst code (without compiling to SVG)
    ///
    /// Usage: export_typst(plot(...), bar(...), title = "My Plot")
    /// Returns: String containing complete Typst code
    pub(crate) fn builtin_export_typst(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::{export_diagram_typst, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("export_typst() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // Parse arguments (same logic as builtin_diagram)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        // Check first arg for options record
        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        // Check last arg for options record (v0.96 named arguments)
        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        // Collect plot elements
        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation { name, .. } = &evaluated {
                if name == "PlotElement" {
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    continue;
                } else {
                    return Err(format!(
                        "export_typst() expects PlotElement, got: {}()",
                        name
                    ));
                }
            }
        }

        if elements.is_empty() {
            return Err("export_typst() requires at least one plot element".to_string());
        }

        // Generate Typst code (without compiling)
        let typst_code = export_diagram_typst(&elements, &options);

        // Return as string
        Ok(Some(Expression::String(typst_code)))
    }

    /// Export just the lq.diagram(...) fragment (without preamble)
    ///
    /// Useful for embedding in existing Typst documents
    pub(crate) fn builtin_export_typst_fragment(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::{export_diagram_typst_fragment, DiagramOptions, PlotElement};

        if args.is_empty() {
            return Err("export_typst_fragment() requires at least one plot element".to_string());
        }

        let mut options = DiagramOptions::default();
        let mut elements: Vec<PlotElement> = Vec::new();

        // Parse arguments (same logic as builtin_diagram)
        let mut start_idx = 0;
        let mut end_idx = args.len();

        if let Some(Expression::Operation {
            name, args: opts, ..
        }) = args.first()
        {
            if name == "record" {
                self.parse_diagram_options(opts, &mut options)?;
                start_idx = 1;
            }
        }

        if end_idx > start_idx {
            if let Some(Expression::Operation {
                name, args: opts, ..
            }) = args.last()
            {
                if name == "record" {
                    self.parse_diagram_options(opts, &mut options)?;
                    end_idx = args.len() - 1;
                }
            }
        }

        for arg in &args[start_idx..end_idx] {
            let evaluated = self.eval_concrete(arg)?;

            if let Expression::List(list_elements) = &evaluated {
                for list_elem in list_elements {
                    if let Expression::Operation { name, .. } = list_elem {
                        if name == "PlotElement" {
                            let element = self.decode_plot_element(list_elem)?;
                            elements.push(element);
                        }
                    }
                }
                continue;
            }

            if let Expression::Operation { name, .. } = &evaluated {
                if name == "PlotElement" {
                    let element = self.decode_plot_element(&evaluated)?;
                    elements.push(element);
                } else if name == "record" {
                    continue;
                }
            }
        }

        if elements.is_empty() {
            return Err("export_typst_fragment() requires at least one plot element".to_string());
        }

        // Generate Typst fragment (without preamble)
        let typst_code = export_diagram_typst_fragment(&elements, &options);

        Ok(Some(Expression::String(typst_code)))
    }

    /// Generate Typst table code from Kleis data
    ///
    /// Usage: table_typst(headers, rows)
    /// - headers: List of column header strings ["Name", "Age", "Score"]
    /// - rows: List of rows, each row is a list [[a, b, c], [d, e, f]]
    ///
    /// Returns: String containing Typst table code
    pub(crate) fn builtin_table_typst(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err(
                "table_typst() requires 2 arguments: headers (list), rows (list of lists)"
                    .to_string(),
            );
        }

        // Extract headers
        let headers_expr = self.eval_concrete(&args[0])?;
        let headers: Vec<String> = match headers_expr {
            Expression::List(items) => items
                .iter()
                .map(|item| match item {
                    Expression::String(s) => s.clone(),
                    Expression::Const(s) => s.clone(),
                    other => format!("{:?}", other),
                })
                .collect(),
            _ => return Err("table_typst(): first argument must be a list of headers".to_string()),
        };

        // Extract rows
        let rows_expr = self.eval_concrete(&args[1])?;
        let rows: Vec<Vec<String>> =
            match rows_expr {
                Expression::List(row_items) => row_items
                    .iter()
                    .map(|row| match row {
                        Expression::List(cells) => cells
                            .iter()
                            .map(|cell| match cell {
                                Expression::String(s) => s.clone(),
                                Expression::Const(s) => s.clone(),
                                other => format!("{:?}", other),
                            })
                            .collect(),
                        _ => vec![format!("{:?}", row)],
                    })
                    .collect(),
                _ => return Err(
                    "table_typst(): second argument must be a list of rows (each row is a list)"
                        .to_string(),
                ),
            };

        // Build Typst table code (no # prefix - for embedding in figures)
        let num_cols = headers.len();
        let mut code = format!("table(\n  columns: {},\n", num_cols);

        // Add headers
        for (i, header) in headers.iter().enumerate() {
            code.push_str(&format!("  [{}]", header));
            if i < num_cols - 1 {
                code.push_str(", ");
            }
        }
        code.push_str(",\n");

        // Add rows
        for row in &rows {
            code.push_str("  ");
            for (i, cell) in row.iter().enumerate() {
                code.push_str(&format!("[{}]", cell));
                if i < row.len() - 1 {
                    code.push_str(", ");
                }
            }
            code.push_str(",\n");
        }

        code.push(')');

        Ok(Some(Expression::String(code)))
    }

    /// Generate Typst table code (raw Object) from Kleis data (no quotes, no '#')
    ///
    /// Usage: table_typst_raw(headers, rows)
    /// - headers: List of column header strings ["Name", "Age", "Score"]
    /// - rows: List of rows, each row is a list [[a, b, c], [d, e, f]]
    ///
    /// Returns: Object containing Typst table code (no string quotes, no #)
    pub(crate) fn builtin_table_typst_raw(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err(
                "table_typst_raw() requires 2 arguments: headers (list), rows (list of lists)"
                    .to_string(),
            );
        }

        // Extract headers as strict strings
        let headers_expr = self.eval_concrete(&args[0])?;
        let headers: Vec<String> = match headers_expr {
            Expression::List(items) => items
                .iter()
                .map(|item| self.extract_string(item))
                .collect::<Result<_, _>>()
                .map_err(|e| format!("table_typst_raw headers: {}", e))?,
            _ => {
                return Err(
                    "table_typst_raw(): first argument must be a list of headers".to_string(),
                )
            }
        };

        // Extract rows as list of list of strings
        let rows_expr = self.eval_concrete(&args[1])?;
        let rows: Vec<Vec<String>> = match rows_expr {
            Expression::List(row_items) => row_items
                .iter()
                .map(|row| match row {
                    Expression::List(cells) => cells
                        .iter()
                        .map(|cell| self.extract_string(cell))
                        .collect::<Result<_, _>>()
                        .map_err(|e| format!("table_typst_raw row: {}", e)),
                    _ => Err("Each row must be a list".to_string()),
                })
                .collect::<Result<_, _>>()?,
            _ => {
                return Err("table_typst_raw(): second argument must be list of rows (list)".into())
            }
        };

        let num_cols = headers.len();
        let mut code = format!("table(\n  columns: {},\n", num_cols);

        // Headers
        for (i, header) in headers.iter().enumerate() {
            code.push_str(&format!("  [{}]", header));
            if i < num_cols - 1 {
                code.push_str(", ");
            } else {
                code.push_str(",\n");
            }
        }

        // Rows
        for row in &rows {
            code.push_str("  [");
            for (i, cell) in row.iter().enumerate() {
                code.push_str(&format!("[{}]", cell));
                if i < num_cols - 1 {
                    code.push_str(", ");
                }
            }
            code.push_str("],\n");
        }

        code.push(')');

        Ok(Some(Expression::Object(code)))
    }

    /// Convert a Typst string to a raw object (no quotes/escapes)
    ///
    /// Usage: typst_raw(text_string)
    /// Returns: Object containing the text verbatim
    pub(crate) fn builtin_typst_raw(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() != 1 {
            return Ok(None);
        }
        let v = self.eval_concrete(&args[0])?;
        match v {
            Expression::String(s) | Expression::Const(s) | Expression::Object(s) => {
                Ok(Some(Expression::Object(s)))
            }
            other => Err(format!(
                "typst_raw(): expected string/object, got {:?}",
                other
            )),
        }
    }

    /// Render an EditorNode AST to Typst code
    ///
    /// Usage: render_to_typst(ast)
    /// Usage: render_to_typst(ast, "typst")  // or "latex", "unicode"
    /// Returns: String containing Typst (or other format) code
    ///
    /// The ast should be a Kleis EditorNode expression, e.g.:
    ///   binop("equals", sym("E"), binop("times", sym("m"), sup(sym("c"), num("2"))))
    pub(crate) fn builtin_render_to_typst(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        use crate::render::RenderTarget;
        use crate::render_editor::render_editor_node;

        if args.is_empty() {
            return Err("render_to_typst() requires an EditorNode argument".to_string());
        }

        // First argument is the EditorNode AST
        let ast_expr = self.eval_concrete(&args[0])?;

        // Optional second argument: render target (default: Typst)
        let target = if args.len() > 1 {
            let target_str = self.extract_string(&args[1])?;
            match target_str.to_lowercase().as_str() {
                "typst" => RenderTarget::Typst,
                "latex" => RenderTarget::LaTeX,
                "unicode" => RenderTarget::Unicode,
                "html" => RenderTarget::HTML,
                "kleis" => RenderTarget::Kleis,
                _ => {
                    return Err(format!(
                        "Unknown render target: '{}'. Use 'typst', 'latex', 'unicode', 'html', or 'kleis'",
                        target_str
                    ))
                }
            }
        } else {
            RenderTarget::Typst
        };

        // Convert Kleis Expression to Rust EditorNode
        let editor_node = self.expression_to_editor_node(&ast_expr)?;

        // Render to the target format
        let output = render_editor_node(&editor_node, &target);

        Ok(Some(Expression::String(output)))
    }

    /// Convert a Kleis EditorNode expression to a Rust EditorNode struct
    ///
    /// Mapping:
    ///   EObject(symbol)           -> EditorNode::Object { object: symbol }
    ///   EConst(value)             -> EditorNode::Const { value }
    ///   EOp(name, args, kind, _)  -> EditorNode::Operation { ... }
    ///   EList(nodes)              -> EditorNode::List { list }
    ///   EPlaceholder(data)        -> EditorNode::Placeholder { ... }
    pub(crate) fn expression_to_editor_node(
        &self,
        expr: &Expression,
    ) -> Result<crate::editor_ast::EditorNode, String> {
        use crate::editor_ast::{EditorNode, OperationData, PlaceholderData};

        match expr {
            // EObject(symbol) -> Object { object: symbol }
            Expression::Operation { name, args, .. } if name == "EObject" => {
                if args.len() != 1 {
                    return Err("EObject expects 1 argument".to_string());
                }
                let symbol = self.extract_string(&args[0])?;
                Ok(EditorNode::Object { object: symbol })
            }

            // EConst(value) -> Const { value }
            Expression::Operation { name, args, .. } if name == "EConst" => {
                if args.len() != 1 {
                    return Err("EConst expects 1 argument".to_string());
                }
                let value = self.extract_string(&args[0])?;
                Ok(EditorNode::Const { value })
            }

            // EPlaceholder(data) -> Placeholder { placeholder }
            Expression::Operation { name, args, .. } if name == "EPlaceholder" => {
                // For now, create a simple placeholder with id 0
                let id = if !args.is_empty() {
                    if let Expression::Const(s) = &args[0] {
                        s.parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };
                Ok(EditorNode::Placeholder {
                    placeholder: PlaceholderData { id, hint: None },
                })
            }

            // EOp(name, args, kind, meta) -> Operation { operation: OperationData }
            Expression::Operation { name, args, .. } if name == "EOp" => {
                if args.len() < 2 {
                    return Err("EOp expects at least 2 arguments (name, args)".to_string());
                }

                let op_name = self.extract_string(&args[0])?;

                // Convert args list
                let op_args = self.extract_editor_node_list(&args[1])?;

                // Optional kind (3rd arg)
                let kind = if args.len() > 2 {
                    let k = self.extract_string(&args[2]).unwrap_or_default();
                    if k.is_empty() || k == "NoMeta" {
                        None
                    } else {
                        Some(k)
                    }
                } else {
                    None
                };

                // Metadata (4th arg) - for now, ignore complex metadata
                // TODO: Parse TensorMeta, MatrixMeta if needed

                Ok(EditorNode::Operation {
                    operation: OperationData {
                        name: op_name,
                        args: op_args,
                        kind,
                        metadata: None,
                    },
                })
            }

            // EList(nodes) -> List { list }
            Expression::Operation { name, args, .. } if name == "EList" => {
                if args.len() != 1 {
                    return Err("EList expects 1 argument (list of nodes)".to_string());
                }
                let nodes = self.extract_editor_node_list(&args[0])?;
                Ok(EditorNode::List { list: nodes })
            }

            // Handle raw Object as a symbol (for convenience)
            Expression::Object(s) => Ok(EditorNode::Object { object: s.clone() }),

            // Handle raw Const as a constant
            Expression::Const(s) => Ok(EditorNode::Const { value: s.clone() }),

            // Handle raw String as a constant
            Expression::String(s) => Ok(EditorNode::Const { value: s.clone() }),

            _ => Err(format!(
                "Cannot convert expression to EditorNode: {:?}",
                expr
            )),
        }
    }

    /// Extract a list of EditorNodes from a Kleis List expression
    pub(crate) fn extract_editor_node_list(
        &self,
        expr: &Expression,
    ) -> Result<Vec<crate::editor_ast::EditorNode>, String> {
        match expr {
            Expression::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    result.push(self.expression_to_editor_node(item)?);
                }
                Ok(result)
            }

            // Handle Cons/Nil list representation
            Expression::Operation { name, args, .. } if name == "Cons" => {
                if args.len() != 2 {
                    return Err("Cons expects 2 arguments".to_string());
                }
                let head = self.expression_to_editor_node(&args[0])?;
                let mut tail = self.extract_editor_node_list(&args[1])?;
                let mut result = vec![head];
                result.append(&mut tail);
                Ok(result)
            }

            Expression::Operation { name, .. } if name == "Nil" => Ok(vec![]),

            Expression::Object(s) if s == "Nil" => Ok(vec![]),

            _ => Err(format!("Expected list of EditorNodes, got: {:?}", expr)),
        }
    }

    /// Parse diagram options from a record expression
    pub(crate) fn parse_diagram_options(
        &self,
        opts: &[Expression],
        options: &mut crate::plotting::DiagramOptions,
    ) -> Result<(), String> {
        for opt in opts {
            if let Expression::Operation { name, args, .. } = opt {
                if name == "field" && args.len() == 2 {
                    let key = self.extract_string(&args[0])?;
                    match key.as_str() {
                        "width" => options.width = Some(self.extract_f64(&args[1])?),
                        "height" => options.height = Some(self.extract_f64(&args[1])?),
                        "title" => options.title = Some(self.extract_string(&args[1])?),
                        "xlabel" => options.xlabel = Some(self.extract_string(&args[1])?),
                        "ylabel" => options.ylabel = Some(self.extract_string(&args[1])?),
                        "xscale" => options.xscale = Some(self.extract_string(&args[1])?),
                        "yscale" => options.yscale = Some(self.extract_string(&args[1])?),
                        "legend" | "legend_position" => {
                            options.legend_position = Some(self.extract_string(&args[1])?)
                        }
                        "grid" => options.grid = Some(self.extract_bool(&args[1])?),
                        "fill" => options.fill = Some(self.extract_string(&args[1])?),
                        "aspect_ratio" => options.aspect_ratio = Some(self.extract_f64(&args[1])?),
                        "xaxis_subticks" | "x_subticks" => {
                            options.xaxis_subticks = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_subticks" | "y_subticks" => {
                            options.yaxis_subticks = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_mirror" | "y_mirror" => {
                            options.yaxis_mirror = Some(self.extract_bool(&args[1])?)
                        }
                        "margin_top" => options.margin_top = Some(self.extract_string(&args[1])?),
                        "margin_bottom" => {
                            options.margin_bottom = Some(self.extract_string(&args[1])?)
                        }
                        "margin_left" => options.margin_left = Some(self.extract_string(&args[1])?),
                        "margin_right" => {
                            options.margin_right = Some(self.extract_string(&args[1])?)
                        }
                        "xaxis_ticks" | "x_ticks" => {
                            options.xaxis_ticks = Some(self.extract_string_list(&args[1])?)
                        }
                        "xaxis_tick_rotate" | "x_tick_rotate" => {
                            options.xaxis_tick_rotate = Some(self.extract_f64(&args[1])?)
                        }
                        "xaxis_ticks_none" | "hide_xaxis_ticks" => {
                            options.xaxis_ticks_none = Some(self.extract_bool(&args[1])?)
                        }
                        "yaxis_ticks_none" | "hide_yaxis_ticks" => {
                            options.yaxis_ticks_none = Some(self.extract_bool(&args[1])?)
                        }
                        "xaxis_tick_unit" | "x_tick_unit" => {
                            options.xaxis_tick_unit = Some(self.extract_f64(&args[1])?)
                        }
                        "xaxis_tick_suffix" | "x_tick_suffix" => {
                            options.xaxis_tick_suffix = Some(self.extract_string(&args[1])?)
                        }
                        "yaxis_tick_unit" | "y_tick_unit" => {
                            options.yaxis_tick_unit = Some(self.extract_f64(&args[1])?)
                        }
                        "yaxis_tick_suffix" | "y_tick_suffix" => {
                            options.yaxis_tick_suffix = Some(self.extract_string(&args[1])?)
                        }
                        "xlim" | "x_lim" => {
                            let limits = self.extract_f64_list_from_diagram_option(&args[1])?;
                            if limits.len() >= 2 {
                                options.xlim = Some((limits[0], limits[1]));
                            }
                        }
                        "ylim" | "y_lim" => {
                            let limits = self.extract_f64_list_from_diagram_option(&args[1])?;
                            if limits.len() >= 2 {
                                options.ylim = Some((limits[0], limits[1]));
                            }
                        }
                        "theme" => options.theme = Some(self.extract_string(&args[1])?),
                        _ => {} // Ignore unknown options
                    }
                }
            }
        }
        Ok(())
    }

    /// plot(xs, ys), bar(xs, heights), etc. - Create a PlotElement (not rendered yet)
    pub(crate) fn builtin_plot_element(
        &self,
        args: &[Expression],
        plot_type: crate::plotting::PlotType,
    ) -> Result<Option<Expression>, String> {
        use crate::plotting::PlotType;

        if args.len() < 2 {
            return Err(format!(
                "{}() requires at least 2 arguments: x_data, y_data",
                match plot_type {
                    PlotType::Line => "plot",
                    PlotType::Scatter => "scatter",
                    PlotType::Bar => "bar",
                    PlotType::HBar => "hbar",
                    PlotType::Stem => "stem",
                    PlotType::HStem => "hstem",
                    PlotType::FillBetween => "fill_between",
                    _ => "plot",
                }
            ));
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let y_data = self.extract_number_list_v2(&args[1])?;

        if x_data.len() != y_data.len() {
            return Err(format!(
                "x_data and y_data must have same length (got {} and {})",
                x_data.len(),
                y_data.len()
            ));
        }

        // Build PlotElement expression with encoded data
        let mut element_args = vec![
            Expression::Const(format!("{:?}", plot_type)),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        // Parse options if present
        if args.len() >= 3 {
            element_args.push(args[2].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// boxplot(data...) - Create a boxplot PlotElement
    pub(crate) fn builtin_boxplot_element(
        &self,
        args: &[Expression],
        horizontal: bool,
    ) -> Result<Option<Expression>, String> {
        if args.is_empty() {
            return Err("boxplot() requires at least one dataset".to_string());
        }

        // Extract datasets
        let mut datasets = Vec::new();
        for arg in args {
            let data = self.extract_number_list_v2(arg)?;
            datasets.push(data);
        }

        let plot_type = if horizontal {
            crate::plotting::PlotType::HBoxplot
        } else {
            crate::plotting::PlotType::Boxplot
        };

        // Encode datasets as nested list
        let datasets_expr =
            Expression::List(datasets.iter().map(|d| self.encode_f64_list(d)).collect());

        Ok(Some(Expression::operation(
            "PlotElement",
            vec![Expression::Const(format!("{:?}", plot_type)), datasets_expr],
        )))
    }

    /// heatmap(matrix) or contour(matrix) - Create matrix-based PlotElement
    pub(crate) fn builtin_matrix_element(
        &self,
        args: &[Expression],
        plot_type: crate::plotting::PlotType,
    ) -> Result<Option<Expression>, String> {
        if args.is_empty() {
            return Err("heatmap/contour() requires a matrix".to_string());
        }

        let matrix = self.extract_f64_matrix(&args[0])?;

        // Encode matrix
        let matrix_expr =
            Expression::List(matrix.iter().map(|row| self.encode_f64_list(row)).collect());

        let mut element_args = vec![Expression::Const(format!("{:?}", plot_type)), matrix_expr];

        // Parse options if present
        if args.len() >= 2 {
            element_args.push(args[1].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// quiver(xs, ys, directions) - Create vector field PlotElement
    pub(crate) fn builtin_quiver_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 3 {
            return Err("quiver() requires: x_coords, y_coords, directions".to_string());
        }

        let x_coords = self.extract_number_list_v2(&args[0])?;
        let y_coords = self.extract_number_list_v2(&args[1])?;
        let dir_matrix = self.extract_f64_matrix(&args[2])?;

        // Encode data
        let x_expr = self.encode_f64_list(&x_coords);
        let y_expr = self.encode_f64_list(&y_coords);
        let dir_expr = Expression::List(
            dir_matrix
                .iter()
                .map(|row| self.encode_f64_list(row))
                .collect(),
        );

        let mut element_args = vec![
            Expression::Const("Quiver".to_string()),
            x_expr,
            y_expr,
            dir_expr,
        ];

        // Parse options if present
        if args.len() >= 4 {
            element_args.push(args[3].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// place(x, y, text, align = "top") - Text annotation at coordinates
    pub(crate) fn builtin_place_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 3 {
            return Err("place() requires: x, y, text".to_string());
        }

        let x = self.extract_f64(&args[0])?;
        let y = self.extract_f64(&args[1])?;
        let text = self.extract_string(&args[2])?;

        let mut element_args = vec![
            Expression::Const("Place".to_string()),
            self.encode_f64_list(&[x]),
            self.encode_f64_list(&[y]),
            Expression::String(text),
        ];

        // Parse options if present (trailing record from named arguments)
        if args.len() >= 4 {
            element_args.push(args[3].clone());
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// yaxis(position = "right", label = "...", child_elements...) - Secondary y-axis
    pub(crate) fn builtin_yaxis_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        // yaxis can contain child plot elements and options
        // yaxis(bar(...), plot(...), position = "right", label = "...")

        let mut child_elements: Vec<Expression> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
                if name == "PlotElement" {
                    child_elements.push(evaluated);
                    continue;
                }
            }

            // Try to evaluate as a plot element
            if let Expression::Operation { name, .. } = &arg {
                if name == "PlotElement" {
                    child_elements.push(arg.clone());
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("SecondaryYAxis".to_string()),
            Expression::List(child_elements),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// xaxis(position = "top", label = "...", functions = ("x => k/x", "x => k/x")) - Secondary x-axis
    pub(crate) fn builtin_xaxis_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        // xaxis can contain child plot elements and options
        // xaxis(plot(...), position = "top", label = "Energy (eV)", functions = ...)

        let mut child_elements: Vec<Expression> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
                if name == "PlotElement" {
                    child_elements.push(evaluated);
                    continue;
                }
            }

            // Try to evaluate as a plot element
            if let Expression::Operation { name, .. } = &arg {
                if name == "PlotElement" {
                    child_elements.push(arg.clone());
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("SecondaryXAxis".to_string()),
            Expression::List(child_elements),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// path(points, fill = "blue", closed = true) - Arbitrary path for polygons/fractals
    pub(crate) fn builtin_path_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        // path can take:
        // - A list of (x, y) pairs: path([(0,0), (1,1), (2,0)])
        // - Separate x and y lists: path(xs, ys)
        // - Options: fill, stroke, closed

        let mut x_data: Vec<f64> = Vec::new();
        let mut y_data: Vec<f64> = Vec::new();
        let mut options_record: Option<Expression> = None;

        for arg in args {
            let evaluated = self.eval_concrete(arg)?;

            // Check for options record
            if let Expression::Operation { name, .. } = &evaluated {
                if name == "record" {
                    options_record = Some(evaluated.clone());
                    continue;
                }
            }

            // Check for list of pairs
            if let Expression::List(items) = &evaluated {
                if !items.is_empty() {
                    // Check if first item is a Pair
                    if let Expression::Operation {
                        name,
                        args: pair_args,
                        ..
                    } = self.eval_concrete(&items[0])?
                    {
                        if name == "Pair" && pair_args.len() == 2 {
                            // It's a list of pairs
                            for item in items {
                                if let Expression::Operation {
                                    name: n,
                                    args: p_args,
                                    ..
                                } = self.eval_concrete(item)?
                                {
                                    if n == "Pair" && p_args.len() == 2 {
                                        x_data.push(self.extract_f64(&p_args[0])?);
                                        y_data.push(self.extract_f64(&p_args[1])?);
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    // Otherwise it might be a list of numbers (x or y data)
                    let nums: Result<Vec<f64>, _> =
                        items.iter().map(|e| self.extract_f64(e)).collect();
                    if let Ok(nums) = nums {
                        if x_data.is_empty() {
                            x_data = nums;
                        } else {
                            y_data = nums;
                        }
                    }
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("Path".to_string()),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// fill_between(xs, y1, y2 = ..., fill = ...) - Shaded area between curves
    pub(crate) fn builtin_fill_between_element(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err("fill_between() requires at least x and y data".to_string());
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let y_data = self.extract_number_list_v2(&args[1])?;

        // Check for y2 parameter in options or as third positional arg
        let mut y2_data: Option<Vec<f64>> = None;
        let mut options_record: Option<Expression> = None;

        for (i, arg) in args.iter().enumerate().skip(2) {
            let evaluated = self.eval_concrete(arg)?;
            if let Expression::Operation {
                ref name, ref args, ..
            } = evaluated
            {
                if name == "record" {
                    // Check for y2 in the record
                    for field_arg in args {
                        if let Expression::Operation {
                            name: fname,
                            args: fargs,
                            ..
                        } = field_arg
                        {
                            if fname == "field" && fargs.len() >= 2 {
                                if let Expression::Const(key) = &fargs[0] {
                                    if key == "y2" {
                                        y2_data = Some(self.extract_number_list_v2(&fargs[1])?);
                                    }
                                }
                            }
                        }
                    }
                    options_record = Some(evaluated);
                }
            } else if i == 2 && y2_data.is_none() {
                // Third positional arg might be y2 array
                if let Ok(y2) = self.extract_number_list_v2(&evaluated) {
                    y2_data = Some(y2);
                }
            }
        }

        let mut element_args = vec![
            Expression::Const("FillBetween".to_string()),
            self.encode_f64_list(&x_data),
            self.encode_f64_list(&y_data),
        ];

        // Add y2 if present
        if let Some(ref y2) = y2_data {
            element_args.push(self.encode_f64_list(y2));
        } else {
            element_args.push(Expression::List(vec![])); // Empty placeholder
        }

        if let Some(opts) = options_record {
            element_args.push(opts);
        }

        Ok(Some(Expression::operation("PlotElement", element_args)))
    }

    /// stacked_area(xs, ys1, ys2, ys3, ...) - Create stacked area chart
    pub(crate) fn builtin_stacked_area(
        &self,
        args: &[Expression],
    ) -> Result<Option<Expression>, String> {
        if args.len() < 2 {
            return Err("stacked_area() requires x data and at least one y series".to_string());
        }

        let x_data = self.extract_number_list_v2(&args[0])?;
        let n = x_data.len();

        // Collect all y series
        let mut y_series: Vec<Vec<f64>> = Vec::new();
        for arg in args.iter().skip(1) {
            let evaluated = self.eval_concrete(arg)?;
            // Skip options records
            if let Expression::Operation { ref name, .. } = evaluated {
                if name == "record" {
                    continue;
                }
            }
            if let Ok(ys) = self.extract_number_list_v2(&evaluated) {
                if ys.len() == n {
                    y_series.push(ys);
                }
            }
        }

        if y_series.is_empty() {
            return Err("stacked_area() requires at least one y series".to_string());
        }

        // Compute cumulative sums (stacked values)
        let mut stacked: Vec<Vec<f64>> = vec![vec![0.0; n]]; // Start with zeros
        for ys in &y_series {
            let prev = stacked.last().expect("stacked initialized non-empty");
            let new_stack: Vec<f64> = prev.iter().zip(ys.iter()).map(|(a, b)| a + b).collect();
            stacked.push(new_stack);
        }

        // Create fill-between elements for each layer
        // stacked[i] to stacked[i+1] for i in 0..y_series.len()
        let mut fill_elements: Vec<Expression> = Vec::new();

        // Default colors for stacked areas
        let colors = [
            "#5B8FB9", "#E19F8F", "#B5651D", "#7CB342", "#9C27B0", "#FF9800",
        ];

        for i in 0..y_series.len() {
            let y1 = &stacked[i];
            let y2 = &stacked[i + 1];
            let color = colors[i % colors.len()];

            let element_args = vec![
                Expression::Const("FillBetween".to_string()),
                self.encode_f64_list(&x_data),
                self.encode_f64_list(y1),
                self.encode_f64_list(y2),
                // Options record with fill color
                Expression::operation(
                    "record",
                    vec![Expression::operation(
                        "field",
                        vec![
                            Expression::Const("fill".to_string()),
                            Expression::Const(format!("rgb(\"{}\")", color)),
                        ],
                    )],
                ),
            ];
            fill_elements.push(Expression::operation("PlotElement", element_args));
        }

        // Return as a list of PlotElements
        Ok(Some(Expression::List(fill_elements)))
    }

    /// Encode a list of f64 as an Expression::List
    pub(crate) fn encode_f64_list(&self, data: &[f64]) -> Expression {
        Expression::List(
            data.iter()
                .map(|&v| Expression::Const(v.to_string()))
                .collect(),
        )
    }

    /// Decode a PlotElement expression back to a PlotElement struct
    pub(crate) fn decode_plot_element(
        &self,
        expr: &Expression,
    ) -> Result<crate::plotting::PlotElement, String> {
        use crate::plotting::{PlotElement, PlotElementOptions, PlotType};

        if let Expression::Operation { name, args, .. } = expr {
            if name != "PlotElement" {
                return Err(format!("Expected PlotElement, got {}", name));
            }

            if args.is_empty() {
                return Err("PlotElement has no arguments".to_string());
            }

            // First arg is the type
            let type_str = self.extract_string(&args[0])?;
            let element_type = match type_str.as_str() {
                "Line" => PlotType::Line,
                "Scatter" => PlotType::Scatter,
                "Bar" => PlotType::Bar,
                "HBar" => PlotType::HBar,
                "Stem" => PlotType::Stem,
                "HStem" => PlotType::HStem,
                "FillBetween" => PlotType::FillBetween,
                "Boxplot" => PlotType::Boxplot,
                "HBoxplot" => PlotType::HBoxplot,
                "Colormesh" => PlotType::Colormesh,
                "Contour" => PlotType::Contour,
                "Quiver" => PlotType::Quiver,
                "Place" => PlotType::Place,
                "SecondaryYAxis" => PlotType::SecondaryYAxis,
                "SecondaryXAxis" => PlotType::SecondaryXAxis,
                "Path" => PlotType::Path,
                _ => return Err(format!("Unknown PlotElement type: {}", type_str)),
            };

            let mut element = PlotElement {
                element_type: element_type.clone(),
                x_data: None,
                y_data: None,
                y2_data: None,
                matrix_data: None,
                direction_data: None,
                datasets: None,
                options: PlotElementOptions::default(),
            };

            // Decode based on type
            match element_type {
                PlotType::Line
                | PlotType::Scatter
                | PlotType::Bar
                | PlotType::HBar
                | PlotType::Stem
                | PlotType::HStem => {
                    if args.len() >= 3 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        self.parse_element_options(&args[3], &mut element.options)?;
                    }
                }
                PlotType::FillBetween => {
                    // fill_between(x, y1, y2, options)
                    if args.len() >= 3 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        // arg[3] could be y2 data or options
                        if let Ok(y2) = self.decode_f64_list(&args[3]) {
                            if !y2.is_empty() {
                                element.y2_data = Some(y2);
                            }
                        }
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::Boxplot | PlotType::HBoxplot => {
                    if args.len() >= 2 {
                        element.datasets = Some(self.decode_f64_matrix(&args[1])?);
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::Colormesh | PlotType::Contour => {
                    if args.len() >= 2 {
                        element.matrix_data = Some(self.decode_f64_matrix(&args[1])?);
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::Quiver => {
                    if args.len() >= 4 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                        // Decode directions as matrix, then convert to tuples
                        let dir_matrix = self.decode_f64_matrix(&args[3])?;
                        let directions: Vec<Vec<(f64, f64)>> = dir_matrix
                            .iter()
                            .map(|row| {
                                row.chunks(2)
                                    .map(|chunk| {
                                        if chunk.len() == 2 {
                                            (chunk[0], chunk[1])
                                        } else {
                                            (chunk[0], 0.0)
                                        }
                                    })
                                    .collect()
                            })
                            .collect();
                        element.direction_data = Some(directions);
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::Place => {
                    if args.len() >= 4 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                        // Text is the 4th argument
                        if let Expression::String(text) = &args[3] {
                            element.options.text = Some(text.clone());
                        } else if let Expression::Const(text) = &args[3] {
                            element.options.text = Some(text.clone());
                        }
                    }
                    if args.len() >= 5 {
                        self.parse_element_options(&args[4], &mut element.options)?;
                    }
                }
                PlotType::SecondaryYAxis | PlotType::SecondaryXAxis => {
                    // args[1] is the list of child elements
                    if args.len() >= 2 {
                        if let Expression::List(children) = &args[1] {
                            let mut decoded_children = Vec::new();
                            for child in children {
                                let decoded = self.decode_plot_element(child)?;
                                decoded_children.push(Box::new(decoded));
                            }
                            element.options.children = Some(decoded_children);
                        }
                    }
                    if args.len() >= 3 {
                        self.parse_element_options(&args[2], &mut element.options)?;
                    }
                }
                PlotType::GroupedBars => {}
                PlotType::Path => {
                    // args[1] is x_data, args[2] is y_data
                    if args.len() >= 2 {
                        element.x_data = Some(self.decode_f64_list(&args[1])?);
                    }
                    if args.len() >= 3 {
                        element.y_data = Some(self.decode_f64_list(&args[2])?);
                    }
                    if args.len() >= 4 {
                        self.parse_element_options(&args[3], &mut element.options)?;
                    }
                }
            }

            Ok(element)
        } else {
            Err(format!("Expected PlotElement expression, got: {:?}", expr))
        }
    }

    /// Decode a list of f64 from an Expression::List
    pub(crate) fn decode_f64_list(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        if let Expression::List(items) = expr {
            let mut result = Vec::new();
            for item in items {
                let n = self
                    .as_number(item)
                    .ok_or_else(|| format!("Expected number in list, got: {:?}", item))?;
                result.push(n);
            }
            Ok(result)
        } else {
            Err(format!("Expected list, got: {:?}", expr))
        }
    }

    /// Decode a 2D matrix from nested Expression::List
    pub(crate) fn decode_f64_matrix(&self, expr: &Expression) -> Result<Vec<Vec<f64>>, String> {
        if let Expression::List(rows) = expr {
            let mut result = Vec::new();
            for row in rows {
                result.push(self.decode_f64_list(row)?);
            }
            Ok(result)
        } else {
            Err(format!("Expected matrix (list of lists), got: {:?}", expr))
        }
    }

    /// Parse element options from a record expression
    pub(crate) fn parse_element_options(
        &self,
        expr: &Expression,
        options: &mut crate::plotting::PlotElementOptions,
    ) -> Result<(), String> {
        if let Expression::Operation { name, args, .. } = expr {
            if name == "record" {
                for opt in args {
                    if let Expression::Operation {
                        name: field_name,
                        args: field_args,
                        ..
                    } = opt
                    {
                        if field_name == "field" && field_args.len() == 2 {
                            let key = self.extract_string(&field_args[0])?;
                            match key.as_str() {
                                "label" => {
                                    options.label = Some(self.extract_string(&field_args[1])?)
                                }
                                "color" => {
                                    options.color = Some(self.extract_string(&field_args[1])?)
                                }
                                "stroke" => {
                                    options.stroke = Some(self.extract_string(&field_args[1])?)
                                }
                                "mark" => options.mark = Some(self.extract_string(&field_args[1])?),
                                "mark_size" => {
                                    options.mark_size = Some(self.extract_f64(&field_args[1])?)
                                }
                                "xerr" => {
                                    options.xerr =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "yerr" => {
                                    options.yerr =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "step" => options.step = Some(self.extract_string(&field_args[1])?),
                                "smooth" => {
                                    options.smooth = Some(self.extract_bool(&field_args[1])?)
                                }
                                "every" => {
                                    options.every = Some(self.extract_f64(&field_args[1])? as usize)
                                }
                                "offset" => {
                                    options.offset = Some(self.extract_f64(&field_args[1])?)
                                }
                                "width" => options.width = Some(self.extract_f64(&field_args[1])?),
                                "fill" => options.fill = Some(self.extract_string(&field_args[1])?),
                                "base" => options.base = Some(self.extract_f64(&field_args[1])?),
                                "colormap" | "map" => {
                                    options.colormap = Some(self.extract_string(&field_args[1])?)
                                }
                                "colors" | "color_values" => {
                                    // Per-point color values for scatter plots (floats 0-1)
                                    options.colors =
                                        Some(self.extract_number_list_v2(&field_args[1])?)
                                }
                                "scale" => options.scale = Some(self.extract_f64(&field_args[1])?),
                                "pivot" => {
                                    options.pivot = Some(self.extract_string(&field_args[1])?)
                                }
                                "clip" => options.clip = Some(self.extract_bool(&field_args[1])?),
                                "z_index" => {
                                    options.z_index = Some(self.extract_f64(&field_args[1])? as i32)
                                }
                                "opacity" | "alpha" => {
                                    options.opacity = Some(self.extract_f64(&field_args[1])?)
                                }
                                // place() options
                                "text" => options.text = Some(self.extract_string(&field_args[1])?),
                                "align" => {
                                    options.align = Some(self.extract_string(&field_args[1])?)
                                }
                                "padding" | "pad" => {
                                    options.padding = Some(self.extract_string(&field_args[1])?)
                                }
                                // yaxis() and xaxis() options
                                "position" => {
                                    options.position = Some(self.extract_string(&field_args[1])?)
                                }
                                "axis_label" => {
                                    options.axis_label = Some(self.extract_string(&field_args[1])?)
                                }
                                // xaxis() specific options
                                "tick_distance" => {
                                    options.tick_distance = Some(self.extract_f64(&field_args[1])?)
                                }
                                "exponent" => {
                                    options.exponent =
                                        Some(self.extract_f64(&field_args[1])? as i32)
                                }
                                "axis_offset" => {
                                    options.axis_offset = Some(self.extract_f64(&field_args[1])?)
                                }
                                // path() options
                                "closed" => {
                                    options.closed = Some(self.extract_bool(&field_args[1])?)
                                }
                                "functions" => {
                                    // functions = ("x => k/x", "x => k/x")
                                    // Expect a pair of strings
                                    if let Expression::Operation {
                                        name,
                                        args: fn_args,
                                        ..
                                    } = self.eval_concrete(&field_args[1])?
                                    {
                                        if name == "Pair" && fn_args.len() == 2 {
                                            options.transform_forward =
                                                Some(self.extract_string(&fn_args[0])?);
                                            options.transform_inverse =
                                                Some(self.extract_string(&fn_args[1])?);
                                        }
                                    }
                                }
                                _ => {} // Ignore unknown options
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
