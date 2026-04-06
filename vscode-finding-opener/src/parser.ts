export interface FindingLocation {
  filePath: string;
  line: number;
}

export interface FindingLineMatch extends FindingLocation {
  lineIndex: number;
  sourceLine: string;
}

const SOURCE_LINE_PATTERN = /^Source:\s+(.+):(\d+)\s*$/;

function normalizePath(filePath: string): string {
  return filePath.trim().replace(/^["'`]/, '').replace(/["'`]$/, '');
}

export function parseFindingLocations(text: string): FindingLocation[] {
  return parseFindingLineMatches(text).map(({ filePath, line }) => ({ filePath, line }));
}

export function parseFindingLineMatches(text: string): FindingLineMatch[] {
  const locations: FindingLineMatch[] = [];
  const lines = text.split(/\r?\n/);

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
    const sourceLine = lines[lineIndex] ?? '';
    const match = SOURCE_LINE_PATTERN.exec(sourceLine);
    if (!match) {
      continue;
    }

    const filePath = normalizePath(match[1] ?? '');
    const line = Number.parseInt(match[2] ?? '1', 10);

    if (!filePath || !Number.isFinite(line) || line < 1) {
      continue;
    }
    locations.push({
      filePath,
      line,
      lineIndex,
      sourceLine,
    });
  }

  return locations;
}
