import React from "react";
import {
  Download,
  FileText,
  BarChart3,
  TrendingDown,
  TrendingUp,
  CheckCircle2,
} from "lucide-react";
import { FinancialSummary, ExportResult, formatMinorCurrency } from "../lib/types";

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
  return (
    <div className="reports-workspace">
      {/* Top Export Command Bar */}
      <div className="reports-command-bar">
        <div className="command-bar-info">
          <h3>Deterministic Financial Reports</h3>
          <span className="command-bar-sub">
            Exact minor-unit ledger arithmetic · Provenance-audited vector PDF & RFC 4180 CSV
            exports
          </span>
        </div>

        <div className="export-button-group">
          <button
            className="btn-export-pdf"
            onClick={() => onExportReport("pdf")}
            disabled={isExporting}
          >
            <Download size={13} />
            <span>{isExporting ? "Rendering PDF..." : "Export Source-Traceable PDF"}</span>
          </button>

          <button
            className="btn-export-csv"
            onClick={() => onExportReport("csv")}
            disabled={isExporting}
          >
            <FileText size={13} />
            <span>{isExporting ? "Generating CSV..." : "Export Ledger CSV"}</span>
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
            <span className="metric-label">Total Inflow / Revenue</span>
            <div className="metric-value amount-credit font-mono">
              {formatMinorCurrency(financialSummary.totalIncomeMinor, "USD")}
            </div>
            <span className="metric-subtext">Approved client receivables & payouts</span>
          </div>

          <div className="metric-tile">
            <span className="metric-label">Total Outflow / Expenses</span>
            <div className="metric-value amount-debit font-mono">
              {formatMinorCurrency(financialSummary.totalExpenseMinor, "USD")}
            </div>
            <span className="metric-subtext">Verified vendor debits & invoices</span>
          </div>

          <div className="metric-tile">
            <span className="metric-label">Net Operating Surplus</span>
            <div
              className={`metric-value font-mono ${
                financialSummary.netSavingsMinor >= 0 ? "amount-credit" : "amount-debit"
              }`}
            >
              {formatMinorCurrency(financialSummary.netSavingsMinor, "USD")}
            </div>
            <span className="metric-subtext">Deterministic minor-unit net balance</span>
          </div>

          {financialSummary.momExpenseChangePercent !== undefined && (
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

      {/* Monthly Cash Flow Trend Visualizer (SVG) */}
      {financialSummary && financialSummary.monthlyBreakdown.length > 0 && (
        <div className="reports-chart-card">
          <div className="chart-card-head">
            <BarChart3 size={15} className="icon-cream" />
            <h4>Monthly Cash Flow Trajectory</h4>
            <span className="chart-sub">Minor-unit integer height scaling</span>
          </div>

          <div className="chart-svg-container">
            <svg width="100%" height="160" viewBox="0 0 640 160" className="trend-svg">
              {financialSummary.monthlyBreakdown.map((m, idx) => {
                const maxAmount = Math.max(
                  ...financialSummary.monthlyBreakdown.map((x) =>
                    Math.max(x.totalIncomeMinor, x.totalExpenseMinor)
                  ),
                  1
                );
                const xOffset = 60 + idx * 160;
                const incomeH = Math.max(Math.round((m.totalIncomeMinor / maxAmount) * 110), 4);
                const expenseH = Math.max(Math.round((m.totalExpenseMinor / maxAmount) * 110), 4);

                return (
                  <g key={m.monthKey} className="chart-group">
                    {/* Income Bar (Emerald) */}
                    <rect
                      x={xOffset}
                      y={120 - incomeH}
                      width="32"
                      height={incomeH}
                      fill="#22C55E"
                      rx="2"
                    />
                    <text
                      x={xOffset + 16}
                      y={114 - incomeH}
                      textAnchor="middle"
                      fill="#86EFAC"
                      fontSize="9"
                      fontFamily="monospace"
                    >
                      {formatMinorCurrency(m.totalIncomeMinor, "USD")}
                    </text>

                    {/* Expense Bar (Ruby) */}
                    <rect
                      x={xOffset + 38}
                      y={120 - expenseH}
                      width="32"
                      height={expenseH}
                      fill="#EF4444"
                      rx="2"
                    />
                    <text
                      x={xOffset + 54}
                      y={114 - expenseH}
                      textAnchor="middle"
                      fill="#FCA5A5"
                      fontSize="9"
                      fontFamily="monospace"
                    >
                      {formatMinorCurrency(m.totalExpenseMinor, "USD")}
                    </text>

                    {/* Month Label */}
                    <text
                      x={xOffset + 35}
                      y="142"
                      textAnchor="middle"
                      fill="#739EC9"
                      fontSize="11"
                      fontFamily="sans-serif"
                    >
                      {m.monthKey}
                    </text>
                  </g>
                );
              })}
            </svg>
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
                    {formatMinorCurrency(m.totalIncomeMinor, "USD")}
                  </td>
                  <td className="num-col font-mono amount-debit">
                    {formatMinorCurrency(m.totalExpenseMinor, "USD")}
                  </td>
                  <td
                    className={`num-col font-mono ${
                      m.netCashFlowMinor >= 0 ? "amount-credit" : "amount-debit"
                    }`}
                  >
                    {formatMinorCurrency(m.netCashFlowMinor, "USD")}
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
          <span className="table-head-title">Generated Report Exports & Cryptographic Proofs</span>
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
