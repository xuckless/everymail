import type { CSSProperties } from "react";

// The row index `.stagger` delays an entrance by: 30ms per row (design/README.md).
export const stagger = (index: number) => ({ "--i": index }) as CSSProperties;
