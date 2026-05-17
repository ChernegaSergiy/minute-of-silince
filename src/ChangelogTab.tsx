import { useCallback, useEffect, useRef, useState } from "react";
import { Card, Spinner, Text, Link, makeStyles, shorthands, tokens } from "@fluentui/react-components";
import Markdown from "react-markdown";
import changelogMd from "../CHANGELOG.md?raw";
import { open } from "@tauri-apps/plugin-shell";

const CHANGELOG_PAGE_SIZE = 1;

interface ChangelogCategory {
  name: string;
  items: string[];
}

interface ChangelogVersion {
  version: string;
  date: string;
  categories: ChangelogCategory[];
}

function parseChangelog(md: string): ChangelogVersion[] {
  const versions: ChangelogVersion[] = [];
  const sections = md.split(/\n(?=## \[)/);
  for (const section of sections) {
    const headerMatch = section.match(/^## \[([\d.]+)]\s*-\s*(.+)$/m);
    if (!headerMatch) continue;
    const version = headerMatch[1];
    const date = headerMatch[2].trim();
    const categories: ChangelogCategory[] = [];
    const catSections = section.split(/\n(?=### )/);
    for (const catSection of catSections) {
      const catMatch = catSection.match(/^### (.+)$/m);
      if (!catMatch) continue;
      const name = catMatch[1];
      const items = catSection
        .split("\n")
        .filter((l) => l.startsWith("- "))
        .map((l) => l.replace(/^- /, "").trim());
      if (items.length > 0) {
        categories.push({ name, items });
      }
    }
    versions.push({ version, date, categories });
  }
  return versions;
}

const changelogVersions = parseChangelog(changelogMd);

const useStyles = makeStyles({
  changelogContent: {
    padding: tokens.spacingVerticalL,
  },
  card: {
    marginBottom: tokens.spacingVerticalM,
  },
  changelogCategory: {
    marginTop: tokens.spacingVerticalS,
  },
  changelogCatHeader: {
    textTransform: "uppercase",
    ...shorthands.margin(0, 0, tokens.spacingVerticalXS, 0),
  },
  changelogSpinner: {
    display: "flex",
    justifyContent: "center",
    paddingTop: tokens.spacingVerticalL,
  },
  changelogList: {
    margin: 0,
    paddingLeft: tokens.spacingHorizontalL,
  },
  changelogItem: {
    marginBottom: tokens.spacingVerticalXXS,
    lineHeight: tokens.lineHeightBase200,
  },
  changelogCode: {
    fontSize: tokens.fontSizeBase200,
    backgroundColor: tokens.colorNeutralBackground5,
    padding: `0 ${tokens.spacingHorizontalXXS}`,
    borderRadius: tokens.borderRadiusSmall,
  },
});

export default function ChangelogTab() {
  const styles = useStyles();
  const [changelogCount, setChangelogCount] = useState(CHANGELOG_PAGE_SIZE);
  const [changelogLoading, setChangelogLoading] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const visibleVersions = changelogVersions.slice(0, changelogCount);

  const handleChangelogScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const { scrollTop, scrollHeight, clientHeight } = e.currentTarget;
    if (scrollHeight - scrollTop - clientHeight < 100 && !changelogLoading) {
      setChangelogLoading(true);
      setChangelogCount((c) => Math.min(c + CHANGELOG_PAGE_SIZE, changelogVersions.length));
    }
  }, [changelogLoading]);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el || changelogCount >= changelogVersions.length) return;
    if (el.scrollHeight <= el.clientHeight && !changelogLoading) {
      setChangelogLoading(true);
      setChangelogCount((c) => Math.min(c + CHANGELOG_PAGE_SIZE, changelogVersions.length));
    }
  }, [changelogCount, changelogLoading]);

  useEffect(() => {
    if (!changelogLoading) return;
    const timer = setTimeout(() => setChangelogLoading(false), 500);
    return () => clearTimeout(timer);
  }, [changelogCount, changelogLoading]);

  return (
    <div className={styles.changelogContent}>
      <div ref={scrollRef} onScroll={handleChangelogScroll}>
        {visibleVersions.map((v) => (
          <div key={v.version}>
            <Card className={styles.card}>
              <Text size={200} weight="semibold">
                v{v.version} — {v.date}
              </Text>
              {v.categories.map((cat) => (
                <div key={cat.name} className={styles.changelogCategory}>
                  <Text weight="semibold" size={100} className={styles.changelogCatHeader}>
                    {cat.name}
                  </Text>
                  <ul className={styles.changelogList}>
                    {cat.items.map((item, i) => (
                      <li key={i} className={styles.changelogItem}>
                        <Markdown
                          components={{
                            p: ({ children }) => <Text size={200}>{children}</Text>,
                            code: ({ children }) => <code className={styles.changelogCode}>{children}</code>,
                            a: ({ href, children }) => (
                              <Link onClick={() => href && open(href)}>{children}</Link>
                            ),
                          }}
                        >
                          {item}
                        </Markdown>
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </Card>
          </div>
        ))}
        {changelogLoading && changelogCount < changelogVersions.length && (
          <div className={styles.changelogSpinner}>
            <Spinner size="tiny" />
          </div>
        )}
      </div>
    </div>
  );
}
