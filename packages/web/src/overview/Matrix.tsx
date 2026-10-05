import { cn } from "cn";
import type { ReactNode } from "react";

import { Cell, Column, Row, Table, TableBody, TableHeader } from "../ui/Table";
import type { Matrix as MatrixModel, MatrixCell } from "./overviewModel";

export function Matrix<T>({ label, matrix, renderCell }: MatrixProps<T>) {
  return (
    <div className="max-w-full overflow-x-auto">
      <Table aria-label={label} className="w-auto min-w-80">
        <TableHeader>
          <Column isRowHeader id="row-label">
            <span className="sr-only">Tree</span>
          </Column>
          {matrix.labels.map((columnLabel, column) => (
            <Column key={`column-${String(column)}`} id={`column-${String(column)}`} align="end">
              {columnLabel}
            </Column>
          ))}
        </TableHeader>
        <TableBody>
          {matrix.rows.map((cells, row) => (
            <Row key={`row-${String(row)}`} id={`row-${String(row)}`}>
              <Cell className="text-ink-muted pr-4">{matrix.labels[row]}</Cell>
              {cells.map((cell) => (
                <Cell
                  key={`cell-${String(cell.column)}`}
                  align="end"
                  className={cn(cell.row === cell.column && "bg-pane")}
                >
                  {renderCell(cell)}
                </Cell>
              ))}
            </Row>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

export interface MatrixProps<T> {
  label: string;
  matrix: MatrixModel<T>;
  renderCell: (cell: MatrixCell<T>) => ReactNode;
}
