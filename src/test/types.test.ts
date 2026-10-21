import { describe, it, expect } from "vitest";
import { formatMinorCurrency } from "../lib/types";

describe("Deterministic Currency Formatting", () => {
  it("formats positive USD amounts correctly", () => {
    expect(formatMinorCurrency(4550, "USD")).toBe("$45.50");
    expect(formatMinorCurrency(100, "USD")).toBe("$1.00");
    expect(formatMinorCurrency(5, "USD")).toBe("$0.05");
  });

  it("formats negative USD amounts (expenses/debits) correctly", () => {
    expect(formatMinorCurrency(-1500, "USD")).toBe("-$15.00");
    expect(formatMinorCurrency(-75, "USD")).toBe("-$0.75");
  });

  it("formats multi-currency minor units accurately", () => {
    expect(formatMinorCurrency(125000, "EUR")).toBe("€1,250.00");
    expect(formatMinorCurrency(9999, "GBP")).toBe("£99.99");
    expect(formatMinorCurrency(49900, "INR")).toBe("₹499.00");
  });

  it("handles non-fractional currencies like JPY", () => {
    expect(formatMinorCurrency(5000, "JPY")).toBe("¥5,000");
    expect(formatMinorCurrency(-12000, "JPY")).toBe("-¥12,000");
  });
});
