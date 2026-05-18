import { parse } from "yaml";
import changelogYaml from "../CHANGELOG.yaml?raw";

const CHANGELOG_CATEGORIES = ["added", "changed", "fixed", "removed", "deprecated", "security"] as const;

export type ChangelogCategoryName = (typeof CHANGELOG_CATEGORIES)[number];

export interface ChangelogEntry {
  category: ChangelogCategoryName;
  key: string;
}

export interface ChangelogVersion {
  version: string;
  date: string;
  entries: ChangelogEntry[];
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isChangelogCategoryName(value: unknown): value is ChangelogCategoryName {
  return typeof value === "string" && CHANGELOG_CATEGORIES.includes(value as ChangelogCategoryName);
}

export function parseChangelogYaml(source: string): ChangelogVersion[] {
  const parsed = parse(source);
  if (!Array.isArray(parsed)) {
    throw new Error("CHANGELOG.yaml must contain a list of versions.");
  }

  return parsed.map((versionNode, index) => {
    if (!isObject(versionNode)) {
      throw new Error(`Invalid changelog version entry at index ${index}.`);
    }

    const { version, date, entries } = versionNode as Partial<ChangelogVersion> & { entries?: unknown };
    if (typeof version !== "string" || typeof date !== "string" || !Array.isArray(entries)) {
      throw new Error(`Invalid changelog version entry at index ${index}.`);
    }

    return {
      version,
      date,
      entries: entries.map((entryNode) => {
        if (!isObject(entryNode)) {
          throw new Error(`Invalid changelog entry in version ${version}.`);
        }

        const { category, key } = entryNode as Partial<ChangelogEntry>;
        if (!isChangelogCategoryName(category) || typeof key !== "string") {
          throw new Error(`Invalid changelog entry in version ${version}.`);
        }

        return { category, key };
      }),
    };
  });
}

export const changelogVersions = parseChangelogYaml(changelogYaml);