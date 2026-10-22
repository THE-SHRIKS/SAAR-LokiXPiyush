use sha2::{Digest, Sha256};

/// Escape string for PDF string literal inside parentheses (text)
pub fn escape_pdf_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '\r' | '\n' => out.push(' '),
            // Replace non-ascii chars with nearest ascii or ? to remain standard Type 1 Latin1
            c if (c as u32) < 32 || (c as u32) > 126 => out.push('?'),
            c => out.push(c),
        }
    }
    out
}

pub struct PdfDocument {
    pages: Vec<String>,
    current_page_stream: String,
    page_width: f32,  // 612.0 for Letter
    page_height: f32, // 792.0 for Letter
}

impl PdfDocument {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current_page_stream: String::new(),
            page_width: 612.0,
            page_height: 792.0,
        }
    }

    pub fn new_page(&mut self) {
        if !self.current_page_stream.is_empty() || self.pages.is_empty() {
            let stream = std::mem::take(&mut self.current_page_stream);
            self.pages.push(stream);
        }
    }

    pub fn set_font(&mut self, font_tag: &str, size: f32) {
        self.current_page_stream
            .push_str(&format!("BT /{font_tag} {size:.1} Tf ET\n"));
    }

    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) {
        self.current_page_stream
            .push_str(&format!("{r:.3} {g:.3} {b:.3} rg\n"));
    }

    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) {
        self.current_page_stream
            .push_str(&format!("{r:.3} {g:.3} {b:.3} RG\n"));
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.current_page_stream
            .push_str(&format!("{width:.2} w\n"));
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: bool, stroke: bool) {
        let op = match (fill, stroke) {
            (true, true) => "b",
            (true, false) => "f",
            (false, true) => "s",
            (false, false) => "n",
        };
        self.current_page_stream
            .push_str(&format!("{x:.1} {y:.1} {w:.1} {h:.1} re {op}\n"));
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.current_page_stream
            .push_str(&format!("{x1:.1} {y1:.1} m {x2:.1} {y2:.1} l S\n"));
    }

    pub fn draw_text(&mut self, font_tag: &str, size: f32, x: f32, y: f32, text: &str) {
        let escaped = escape_pdf_text(text);
        self.current_page_stream.push_str(&format!(
            "BT /{font_tag} {size:.1} Tf {x:.1} {y:.1} Td ({escaped}) Tj ET\n"
        ));
    }

    pub fn draw_text_right(&mut self, font_tag: &str, size: f32, right_x: f32, y: f32, text: &str) {
        // Approximate width for standard Helvetica: ~0.55 * size per char
        let approx_width = (text.len() as f32) * size * 0.52;
        let start_x = (right_x - approx_width).max(50.0);
        self.draw_text(font_tag, size, start_x, y, text);
    }

    pub fn page_count(&self) -> usize {
        let mut count = self.pages.len();
        if !self.current_page_stream.is_empty() {
            count += 1;
        }
        count.max(1)
    }

    /// Compile into standards-compliant PDF 1.4 byte stream with exact xref table and trailer
    pub fn build(mut self) -> Vec<u8> {
        if !self.current_page_stream.is_empty() || self.pages.is_empty() {
            let stream = std::mem::take(&mut self.current_page_stream);
            self.pages.push(stream);
        }

        let num_pages = self.pages.len();
        // Object numbering plan:
        // 1 0 obj: Catalog
        // 2 0 obj: Pages
        // For each page i (0-based):
        //   Page object: 3 + i*2
        //   Content stream: 4 + i*2
        // Font F1 (Helvetica): 3 + num_pages*2
        // Font F2 (Helvetica-Bold): 4 + num_pages*2
        let font_f1_id = 3 + num_pages * 2;
        let font_f2_id = 4 + num_pages * 2;
        let total_objs = font_f2_id;

        let mut objects: Vec<(usize, String)> = Vec::new();

        // 1 0 obj: Catalog
        objects.push((1, "<< /Type /Catalog /Pages 2 0 R >>\n".to_string()));

        // 2 0 obj: Pages
        let mut kids = String::new();
        for i in 0..num_pages {
            let page_obj_id = 3 + i * 2;
            kids.push_str(&format!("{page_obj_id} 0 R "));
        }
        objects.push((
            2,
            format!("<< /Type /Pages /Kids [{kids}] /Count {num_pages} >>\n"),
        ));

        // Pages and Content streams
        for i in 0..num_pages {
            let page_obj_id = 3 + i * 2;
            let stream_obj_id = 4 + i * 2;
            let stream_content = &self.pages[i];
            let stream_len = stream_content.len();

            let page_obj = format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.1} {:.1}] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R /F2 {} 0 R >> >> >>\n",
                self.page_width, self.page_height, stream_obj_id, font_f1_id, font_f2_id
            );
            objects.push((page_obj_id, page_obj));

            let stream_obj =
                format!("<< /Length {stream_len} >>\nstream\n{stream_content}endstream\n");
            objects.push((stream_obj_id, stream_obj));
        }

        // Fonts
        objects.push((
            font_f1_id,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n".to_string(),
        ));
        objects.push((
            font_f2_id,
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\n".to_string(),
        ));

        // Sort by object ID
        objects.sort_by_key(|k| k.0);

        // Assemble document
        let header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n";
        let mut doc_bytes = Vec::with_capacity(4096 + num_pages * 2048);
        doc_bytes.extend_from_slice(header);

        let mut offsets: Vec<usize> = vec![0; total_objs + 1];

        for (id, body) in objects {
            offsets[id] = doc_bytes.len();
            let obj_header = format!("{id} 0 obj\n");
            doc_bytes.extend_from_slice(obj_header.as_bytes());
            doc_bytes.extend_from_slice(body.as_bytes());
            doc_bytes.extend_from_slice(b"endobj\n");
        }

        let xref_pos = doc_bytes.len();
        let mut xref = format!("xref\n0 {}\n", total_objs + 1);
        xref.push_str("0000000000 65535 f \n");
        for off in &offsets[1..] {
            xref.push_str(&format!("{off:010} 00000 n \n"));
        }

        let trailer = format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objs + 1,
            xref_pos
        );

        doc_bytes.extend_from_slice(xref.as_bytes());
        doc_bytes.extend_from_slice(trailer.as_bytes());

        doc_bytes
    }
}

#[derive(Debug, Clone)]
pub struct ReconciliationReportRow {
    pub date: String,
    pub description: String,
    pub amount_minor: i64,
    pub currency: String,
    pub match_status: String,
    pub invoice_number: String,
    pub provenance_id: String,
}

pub struct ReportGenerator;

impl ReportGenerator {
    /// Format integer minor units as standard currency string without binary floating point
    pub fn format_minor(cents: i64, currency: &str) -> String {
        let is_negative = cents < 0;
        let abs = cents.abs();
        let whole = abs / 100;
        let frac = abs % 100;
        let sign = if is_negative { "-" } else { "" };
        format!("{sign}{currency} {whole}.{frac:02}")
    }

    pub fn generate_reconciliation_pdf(
        workspace_name: &str,
        period: &str,
        currency: &str,
        total_income_minor: i64,
        total_expense_minor: i64,
        net_savings_minor: i64,
        records: &[ReconciliationReportRow],
    ) -> (Vec<u8>, String) {
        let mut pdf = PdfDocument::new();

        // Calculate rows per page
        // Page 1: Header, Executive Summary box, then up to 14 table rows
        // Subsequent pages: Compact header, then up to 26 table rows
        let rows_page_1 = 14;
        let rows_subsequent = 26;

        let total_records = records.len();
        let total_pages = if total_records <= rows_page_1 {
            1
        } else {
            1 + ((total_records - rows_page_1) + rows_subsequent - 1) / rows_subsequent
        };

        // Compute provenance snapshot SHA-256 of all records
        let mut hasher = Sha256::new();
        hasher.update(workspace_name.as_bytes());
        hasher.update(period.as_bytes());
        hasher.update(currency.as_bytes());
        for r in records {
            hasher.update(r.date.as_bytes());
            hasher.update(r.description.as_bytes());
            hasher.update(&r.amount_minor.to_le_bytes());
            hasher.update(r.match_status.as_bytes());
            hasher.update(r.invoice_number.as_bytes());
            hasher.update(r.provenance_id.as_bytes());
        }
        let snapshot_hash = hex::encode(hasher.finalize());
        let snapshot_short = if snapshot_hash.len() >= 16 {
            &snapshot_hash[..16]
        } else {
            &snapshot_hash
        };

        let mut record_idx = 0;

        for page_num in 1..=total_pages {
            pdf.new_page();

            let mut y;

            if page_num == 1 {
                // Top Brand & Header Banner
                pdf.set_fill_color(0.08, 0.12, 0.22); // Deep navy
                pdf.draw_rect(50.0, 715.0, 512.0, 45.0, true, false);

                pdf.set_fill_color(1.0, 1.0, 1.0); // White text
                pdf.draw_text("F2", 16.0, 60.0, 735.0, "OPENDECK FINANCIAL INTELLIGENCE");
                pdf.draw_text(
                    "F1",
                    10.0,
                    60.0,
                    722.0,
                    "Air-Gapped Offline Reconciliation & Audit Report",
                );

                // Workspace & Period Metadata Subhead
                y = 695.0;
                pdf.set_fill_color(0.2, 0.2, 0.25);
                pdf.draw_text("F2", 10.0, 50.0, y, "Workspace:");
                pdf.draw_text("F1", 10.0, 115.0, y, workspace_name);

                pdf.draw_text("F2", 10.0, 240.0, y, "Reporting Period:");
                pdf.draw_text("F1", 10.0, 335.0, y, period);

                pdf.draw_text("F2", 10.0, 420.0, y, "Currency:");
                pdf.draw_text("F1", 10.0, 475.0, y, currency);

                // Thin divider
                y -= 10.0;
                pdf.set_stroke_color(0.85, 0.87, 0.90);
                pdf.set_line_width(0.75);
                pdf.draw_line(50.0, y, 562.0, y);

                // Executive Summary Card
                y -= 60.0;
                pdf.set_fill_color(0.96, 0.97, 0.99); // Soft slate background
                pdf.draw_rect(50.0, y, 512.0, 55.0, true, false);
                pdf.set_stroke_color(0.80, 0.84, 0.90);
                pdf.set_line_width(1.0);
                pdf.draw_rect(50.0, y, 512.0, 55.0, false, true);

                // Metric 1: Total Inflow
                pdf.set_fill_color(0.4, 0.45, 0.5);
                pdf.draw_text("F2", 8.0, 65.0, y + 38.0, "TOTAL INFLOW");
                pdf.set_fill_color(0.06, 0.48, 0.28); // Green
                let inc_str = Self::format_minor(total_income_minor, currency);
                pdf.draw_text("F2", 12.0, 65.0, y + 20.0, &inc_str);

                // Metric 2: Total Outflow
                pdf.set_fill_color(0.4, 0.45, 0.5);
                pdf.draw_text("F2", 8.0, 195.0, y + 38.0, "TOTAL OUTFLOW");
                pdf.set_fill_color(0.75, 0.15, 0.15); // Crimson
                let exp_str = Self::format_minor(total_expense_minor, currency);
                pdf.draw_text("F2", 12.0, 195.0, y + 20.0, &exp_str);

                // Metric 3: Net Cash Flow
                pdf.set_fill_color(0.4, 0.45, 0.5);
                pdf.draw_text("F2", 8.0, 325.0, y + 38.0, "NET CASH FLOW");
                let net_color = if net_savings_minor >= 0 {
                    (0.06, 0.48, 0.28)
                } else {
                    (0.75, 0.15, 0.15)
                };
                pdf.set_fill_color(net_color.0, net_color.1, net_color.2);
                let net_str = Self::format_minor(net_savings_minor, currency);
                pdf.draw_text("F2", 12.0, 325.0, y + 20.0, &net_str);

                // Metric 4: Reconciled Status
                let approved_count = records
                    .iter()
                    .filter(|r| r.match_status == "APPROVED")
                    .count();
                pdf.set_fill_color(0.4, 0.45, 0.5);
                pdf.draw_text("F2", 8.0, 445.0, y + 38.0, "RECONCILED");
                pdf.set_fill_color(0.08, 0.12, 0.22);
                let recon_str = format!("{approved_count} / {} items", records.len());
                pdf.draw_text("F2", 10.0, 445.0, y + 20.0, &recon_str);

                y -= 25.0;
            } else {
                // Subsequent page header
                pdf.set_fill_color(0.08, 0.12, 0.22);
                pdf.draw_rect(50.0, 740.0, 512.0, 20.0, true, false);
                pdf.set_fill_color(1.0, 1.0, 1.0);
                pdf.draw_text(
                    "F2",
                    9.0,
                    60.0,
                    746.0,
                    "OPENDECK RECONCILIATION REPORT (CONTINUED)",
                );
                let page_meta = format!("Workspace: {workspace_name} | Period: {period}");
                pdf.draw_text_right("F1", 8.0, 555.0, 746.0, &page_meta);

                y = 720.0;
            }

            // Table Header
            pdf.set_fill_color(0.15, 0.20, 0.30);
            pdf.draw_rect(50.0, y - 18.0, 512.0, 18.0, true, false);
            pdf.set_fill_color(1.0, 1.0, 1.0);
            pdf.draw_text("F2", 8.0, 55.0, y - 12.0, "POSTED DATE");
            pdf.draw_text("F2", 8.0, 120.0, y - 12.0, "DESCRIPTION");
            pdf.draw_text("F2", 8.0, 290.0, y - 12.0, "AMOUNT");
            pdf.draw_text("F2", 8.0, 360.0, y - 12.0, "STATUS");
            pdf.draw_text("F2", 8.0, 425.0, y - 12.0, "INVOICE #");
            pdf.draw_text("F2", 8.0, 485.0, y - 12.0, "PROVENANCE");

            y -= 20.0;

            // Table Rows
            let page_limit = if page_num == 1 {
                rows_page_1
            } else {
                rows_subsequent
            };
            let mut page_row_count = 0;

            while record_idx < records.len() && page_row_count < page_limit {
                let rec = &records[record_idx];

                // Zebra striping
                if page_row_count % 2 == 1 {
                    pdf.set_fill_color(0.97, 0.98, 0.99);
                    pdf.draw_rect(50.0, y - 14.0, 512.0, 16.0, true, false);
                }

                // Row borders
                pdf.set_stroke_color(0.92, 0.93, 0.95);
                pdf.set_line_width(0.5);
                pdf.draw_line(50.0, y - 14.0, 562.0, y - 14.0);

                // Date
                pdf.set_fill_color(0.2, 0.2, 0.25);
                pdf.draw_text("F1", 7.5, 55.0, y - 10.0, &rec.date);

                // Description (truncated to 28 chars to fit column)
                let desc_trunc = if rec.description.len() > 28 {
                    format!("{}...", &rec.description[..25])
                } else {
                    rec.description.clone()
                };
                pdf.draw_text("F1", 7.5, 120.0, y - 10.0, &desc_trunc);

                // Amount
                let amt_str = Self::format_minor(rec.amount_minor, &rec.currency);
                let amt_color = if rec.amount_minor >= 0 {
                    (0.06, 0.48, 0.28)
                } else {
                    (0.15, 0.20, 0.25)
                };
                pdf.set_fill_color(amt_color.0, amt_color.1, amt_color.2);
                pdf.draw_text("F2", 7.5, 290.0, y - 10.0, &amt_str);

                // Match Status Badge
                let status_color = match rec.match_status.as_str() {
                    "APPROVED" => (0.06, 0.48, 0.28),
                    "PROPOSED" => (0.80, 0.45, 0.05),
                    "REJECTED" => (0.75, 0.15, 0.15),
                    _ => (0.45, 0.50, 0.55),
                };
                pdf.set_fill_color(status_color.0, status_color.1, status_color.2);
                pdf.draw_text("F2", 7.0, 360.0, y - 10.0, &rec.match_status);

                // Invoice Number
                pdf.set_fill_color(0.2, 0.2, 0.25);
                let inv_str = if rec.invoice_number.is_empty() {
                    "—".to_string()
                } else {
                    rec.invoice_number.clone()
                };
                pdf.draw_text("F1", 7.5, 425.0, y - 10.0, &inv_str);

                // Provenance ID (short SHA or doc id)
                let prov_short = if rec.provenance_id.len() > 10 {
                    &rec.provenance_id[..10]
                } else {
                    &rec.provenance_id
                };
                pdf.set_fill_color(0.4, 0.45, 0.5);
                pdf.draw_text("F1", 7.0, 485.0, y - 10.0, prov_short);

                y -= 16.0;
                record_idx += 1;
                page_row_count += 1;
            }

            // If last page, print statutory notice above footer
            if page_num == total_pages {
                y = (y - 15.0).max(65.0);
                pdf.set_stroke_color(0.85, 0.87, 0.90);
                pdf.set_line_width(0.5);
                pdf.draw_line(50.0, y, 562.0, y);

                y -= 10.0;
                pdf.set_fill_color(0.45, 0.50, 0.55);
                pdf.draw_text(
                    "F1",
                    6.5,
                    50.0,
                    y,
                    "PROVENANCE & AIR-GAP GUARANTEE: All totals computed deterministically in 64-bit integer minor units.",
                );
                y -= 8.0;
                pdf.draw_text(
                    "F1",
                    6.5,
                    50.0,
                    y,
                    "Strict air-gap active: zero cloud calls, zero telemetry. Verify against audited originals before statutory tax filing.",
                );
            }

            // Footer on every page
            pdf.set_stroke_color(0.85, 0.87, 0.90);
            pdf.set_line_width(0.5);
            pdf.draw_line(50.0, 42.0, 562.0, 42.0);

            pdf.set_fill_color(0.45, 0.50, 0.55);
            let footer_left = format!("OpenDeck Offline Intelligence | Snapshot: {snapshot_short}");
            pdf.draw_text("F1", 7.0, 50.0, 32.0, &footer_left);

            let footer_right = format!("Page {page_num} of {total_pages}");
            pdf.draw_text_right("F1", 7.0, 562.0, 32.0, &footer_right);
        }

        let pdf_bytes = pdf.build();
        (pdf_bytes, snapshot_hash)
    }
}
