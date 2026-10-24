import React from "react";
import {
  Download,
  FileText,
  BarChart3,
  TrendingDown,
  TrendingUp,
  CheckCircle2,
} from "lucide-react";
import { CurrencyCode, FinancialSummary, ExportResult, formatMinorCurrency } from "../lib/types";

interface ReportsViewProps {
  financialSummary: FinancialSummary | null;
  reportExports: ExportResult[];
  isExporting: boolean;
  exportFeedback: string | null;
  onExportReport: (format: "pdf" | "csv") => Promise<void>;
}

export const ReportsView: React.FC<ReportsViewProps> = ({
  financialSummary,
  reportExports,
  isExporting,
  exportFeedback,
  onExportReport,
}) => {
  // Amounts are shown in the currency they are stored in, never assumed to be USD.
  const baseCurrency = (financialSummary?.currency ?? "USD") as CurrencyCode;
  return (
    <div className="reports-workspace">
      {/* Top Export Command Bar */}
      <div className="reports-command-bar">
        <div className="command-bar-info">
          <h3>Reports</h3>
          <span className="command-bar-sub">
            Totals are calculated from your records. Export as PDF or CSV.
          </span>
        </div>

        <div className="export-button-group">
          <button
            className="btn-export-pdf"
            onClick={() => onExportReport("pdf")}
            disabled={isExporting}
          >
            <Download size={13} />
            <span>{isExporting ? "Rendering PDF..." : "Export PDF"}</span>
          </button>

          <button
            className="btn-export-csv"
            onClick={() => onExportReport("csv")}
            disabled={isExporting}
          >
            <FileText size={13} />
            <span>{isExporting ? "Generating CSV..." : "Export CSV"}</span>
          </button>
        </div>
      </div>

      {exportFeedback && (
        <div className="export-feedback-banner">
          <CheckCircle2 size={14} className="icon-green" />
          <span className="font-mono text-xs">{exportFeedback}</span>
        </div>
      )}

      {/* KPI Metric Cards Grid */}
      {financialSummary && (
        <div className="reports-metric-grid">
          <div className="metric-tile">
            <span className="metric-label">Money in</span>
            <div className="metric-value amount-credit font-mono">
              {formatMinorCurrency(financialSummary.totalIncomeMinor, baseCurrency)}
            </div>
            <span className="metric-subtext">Credits in {financialSummary.currency}, all imported records</span>
          </div>

          <div className="metric-tile">
            <span className="metric-label">Money out</span>
            <div className="metric-value amount-debit font-mono">
              {formatMinorCurrency(financialSummary.totalExpenseMinor, baseCurrency)}
            </div>
            <span className="metric-subtext">Debits in {financialSummary.currency}, all imported records</span>
          </div>

          <div className="metric-tile">
            <span className="metric-label">Net</span>
            <div
              className={`metric-value font-mono ${
                financialSummary.netSavingsMinor >= 0 ? "amount-credit" : "amount-debit"
              }`}
            >
              {formatMinorCurrency(financialSummary.netSavingsMinor, baseCurrency)}
            </div>
            <span className="metric-subtext">Credits minus debits in {financialSummary.currency}</span>
          </div>

          {financialSummary.momExpenseChangePercent != null && (
            <div className="metric-tile">
              <span className="metric-label">MoM Expense Trend</span>
              <div className="metric-trend-row">
                <span className="metric-value font-mono">
                  {financialSummary.momExpenseChangePercent > 0 ? "+" : ""}
                  {financialSummary.momExpenseChangePercent.toFixed(1)}%
                </span>
                {financialSummary.momExpenseChangePercent <= 0 ? (
                  <TrendingDown size={18} className="icon-green" />
                ) : (
                  <TrendingUp size={18} className="icon-danger" />
                )}
              </div>
              <span className="metric-subtext">
                {financialSummary.momExpenseChangePercent <= 0
                  ? "Expense reduction"
                  : "Expense increase"}{" "}
                vs prior cycle
              </span>
            </div>
          )}
        </div>
      )}

      {/* Transactions in other currencies are never merged into these totals; say so. */}
      {financialSummary && financialSummary.excludedCurrencies && financialSummary.excludedCurrencies.length > 0 && (
        <div className="currency-exclusion-notice" role="note">
          <strong>Not included in these totals:</strong>{" "}
          {financialSummary.excludedCurrencies
            .map(
              (c) =>
                `${c.transactionCount} transaction${c.transactionCount === 1 ? "" : "s"} in ${c.currency} (${formatMinorCurrency(c.grossMinor, c.currency as CurrencyCode)} gross)`,
            )
            .join("; ")}
          . They are in a different currency and no exchange rate is recorded.
        </div>
      )}

      {/* Monthly Cash Flow Trend Visualizer (SVG) */}
      {financialSummary && financialSummary.monthlyBreakdown.length > 0 && (
        <div className="reports-chart-card">
          <div className="chart-card-head">
            <BarChart3 size={15} className="icon-cream" />
            <h4>Monthly cash flow</h4>
            <span className="chart-sub">Income and expenses by month</span>
          </div>

          <div className="chart-svg-container">
            {(() => {
              const months = financialSummary.monthlyBreakdown;
              const maxAmount = Math.max(...months.flatMap((x) => [x.totalIncomeMinor, x.totalExpenseMinor]), 1);
              // Slots scale with the number of months, so every month is drawn.
              const plotLeft = 20;
              const plotWidth = 600;
              const slot = plotWidth / months.length;
              const barW = Math.max(6, Math.min(32, slot / 3));
              const baseline = 158;
              const maxBar = 112;
              const symbolOf = (cur: CurrencyCode) => formatMinorCurrency(0, cur).replace(/[0-9.,\s]/g, "");
              const compact = (minor: number, cur: CurrencyCode) => {
                const major = Math.round(Math.abs(minor) / 100);
                const body =
                  major >= 1_000_000 ? `${(major / 1_000_000).toFixed(1)}M` : major >= 1_000 ? `${(major / 1_000).toFixed(1)}k` : `${major}`;
                return `${symbolOf(cur)}${body}`;
              };
              const thin = months.length > 8; // with many months, label every other one
              return (
                <svg width="100%" height="200" viewBox="0 0 640 200" className="trend-svg" role="img" aria-label="Monthly income and expenses">
                  {months.map((m, idx) => {
                    const cur = m.currency as CurrencyCode;
                    const centre = plotLeft + idx * slot + slot / 2;
                    const incomeH = Math.max(Math.round((m.totalIncomeMinor / maxAmount) * maxBar), 2);
                    const expenseH = Math.max(Math.round((m.totalExpenseMinor / maxAmount) * maxBar), 2);
                    return (
                      <g key={m.monthKey} className="chart-group">
                        <title>{`${m.monthKey}: income ${formatMinorCurrency(m.totalIncomeMinor, cur)}, expenses ${formatMinorCurrency(m.totalExpenseMinor, cur)}`}</title>
                        <rect x={centre - barW - 2} y={baseline - incomeH} width={barW} height={incomeH} style={{ fill: "var(--color-credit)" }} rx="2" />
                        <rect x={centre + 2} y={baseline - expenseH} width={barW} height={expenseH} style={{ fill: "var(--color-debit)" }} rx="2" />
                        {/* Income label ends at the centre line, expense label starts at it: they never overlap. */}
                        <text x={centre - 3} y={baseline - incomeH - 5} textAnchor="end" style={{ fill: "var(--text-secondary)" }} fontSize="10" fontFamily="monospace">
                          {compact(m.totalIncomeMinor, cur)}
                        </text>
                        <text x={centre + 3} y={baseline - expenseH - 5} textAnchor="start" style={{ fill: "var(--text-secondary)" }} fontSize="10" fontFamily="monospace">
                          {compact(m.totalExpenseMinor, cur)}
                        </text>
                        {(!thin || idx % 2 === 0) && (
                          <text x={centre} y={baseline + 18} textAnchor="middle" style={{ fill: "var(--text-muted)" }} fontSize="11" fontFamily="sans-serif">
                            {m.monthKey}
                          </text>
                        )}
                      </g>
                    );
                  })}
                </svg>
              );
            })()}
          </div>
        </div>
      )}

      {/* Monthly Breakdown Table */}
      {financialSummary && (
        <div className="reports-table-card">
          <div className="table-card-head">
            <span className="table-head-title">Monthly Ledger Breakdown</span>
          </div>

          <table className="app-data-table">
            <thead>
              <tr>
                <th>Period</th>
                <th>Transactions</th>
                <th className="num-col">Total Inflow</th>
                <th className="num-col">Total Outflow</th>
                <th className="num-col">Net Cash Flow</th>
              </tr>
            </thead>
            <tbody>
              {financialSummary.monthlyBreakdown.map((m) => (
                <tr key={m.monthKey} className="table-row">
                  <td className="font-mono text-xs">{m.monthKey}</td>
                  <td className="font-mono text-xs">{m.transactionCount}</td>
                  <td className="num-col font-mono amount-credit">
                    {formatMinorCurrency(m.totalIncomeMinor, m.currency as CurrencyCode)}
                  </td>
                  <td className="num-col font-mono amount-debit">
                    {formatMinorCurrency(m.totalExpenseMinor, m.currency as CurrencyCode)}
                  </td>
                  <td
                    className={`num-col font-mono ${
                      m.netCashFlowMinor >= 0 ? "amount-credit" : "amount-debit"
                    }`}
                  >
                    {formatMinorCurrency(m.netCashFlowMinor, m.currency as CurrencyCode)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Historical Report Exports Table */}
      <div className="reports-table-card">
        <div className="table-card-head">
          <span className="table-head-title">Exported reports</span>
          <span className="table-head-subtitle">SHA-256 immutable snapshot fingerprints</span>
        </div>

        {reportExports.length === 0 ? (
          <div className="empty-panel-state">
            <p>No reports exported yet.</p>
            <span>
              Click &ldquo;Export Source-Traceable PDF&rdquo; above to generate your first audit
              package.
            </span>
          </div>
        ) : (
          <table className="app-data-table">
            <thead>
              <tr>
                <th>Exported At</th>
                <th>Format</th>
                <th>File Location</th>
                <th>Size</th>
                <th>Record Snapshot Hash</th>
              </tr>
            </thead>
            <tbody>
              {reportExports.map((exp) => (
                <tr key={exp.exportId} className="table-row">
                  <td className="font-mono text-muted text-xs">{exp.createdAt}</td>
                  <td>
                    <span className="doc-type-badge">{exp.exportFormat}</span>
                  </td>
                  <td className="font-mono text-xs path-cell">{exp.filePath}</td>
                  <td className="font-mono text-xs">{(exp.fileSizeBytes / 1024).toFixed(1)} KB</td>
                  <td className="font-mono text-muted text-xs">
                    {exp.recordSnapshotHash.slice(0, 16)}...
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
};
