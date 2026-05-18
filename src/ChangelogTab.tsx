import { useCallback, useEffect, useRef, useState } from "react";
import { Card, Spinner, Text, Link, makeStyles, shorthands, tokens } from "@fluentui/react-components";
import Markdown from "react-markdown";
import { changelogVersions } from "./changelog";
import { t } from "./i18n";
import { open } from "@tauri-apps/plugin-shell";

const CHANGELOG_PAGE_SIZE = 1;

type ChangelogLocale = Record<string, Record<string, string>> & {
  categories?: Record<string, string>;
};

type ChangelogCategory = {
  category: string;
  keys: string[];
};

type ChangelogViewVersion = {
  version: string;
  date: string;
  categories: ChangelogCategory[];
};

const changelogLocale = t("changelog", { returnObjects: true }) as ChangelogLocale | string;
const changelogTexts = typeof changelogLocale === "string" ? undefined : changelogLocale;

function groupChangelogVersions(): ChangelogViewVersion[] {
  return changelogVersions.map((version) => {
    const categoryMap = new Map<string, string[]>();
    const categories: ChangelogCategory[] = [];

    for (const entry of version.entries) {
      let keys = categoryMap.get(entry.category);
      if (!keys) {
        keys = [];
        categoryMap.set(entry.category, keys);
        categories.push({ category: entry.category, keys });
      }

      keys.push(entry.key);
    }

    return {
      version: version.version,
      date: version.date,
      categories,
    };
  });
}

const groupedChangelogVersions = groupChangelogVersions();

const useStyles = makeStyles({
  changelogContent: {
    height: "100%",
    overflowY: "auto",
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

  const visibleVersions = groupedChangelogVersions.slice(0, changelogCount);

  const handleChangelogScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const { scrollTop, scrollHeight, clientHeight } = e.currentTarget;
    if (scrollHeight - scrollTop - clientHeight < 100 && !changelogLoading) {
      setChangelogLoading(true);
      setChangelogCount((c) => Math.min(c + CHANGELOG_PAGE_SIZE, groupedChangelogVersions.length));
    }
  }, [changelogLoading]);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el || changelogCount >= groupedChangelogVersions.length) return;
    if (el.scrollHeight <= el.clientHeight && !changelogLoading) {
      setChangelogLoading(true);
      setChangelogCount((c) => Math.min(c + CHANGELOG_PAGE_SIZE, groupedChangelogVersions.length));
    }
  }, [changelogCount, changelogLoading]);

  useEffect(() => {
    if (!changelogLoading) return;
    const timer = setTimeout(() => setChangelogLoading(false), 500);
    return () => clearTimeout(timer);
  }, [changelogCount, changelogLoading]);

  return (
    <div ref={scrollRef} className={styles.changelogContent} onScroll={handleChangelogScroll}>
      {visibleVersions.map((v) => (
        <div key={v.version}>
          <Card className={styles.card}>
            <Text size={200} weight="semibold">
              v{v.version} — {v.date}
            </Text>
            {v.categories.map((cat) => (
              <div key={cat.category} className={styles.changelogCategory}>
                <Text weight="semibold" size={100} className={styles.changelogCatHeader}>
                  {changelogTexts?.categories?.[cat.category] ?? cat.category}
                </Text>
                <ul className={styles.changelogList}>
                  {cat.keys.map((itemKey, i) => {
                    const item = changelogTexts?.[v.version]?.[itemKey] ?? itemKey;

                    return (
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
                    );
                  })}
                </ul>
              </div>
            ))}
          </Card>
        </div>
      ))}
      {changelogLoading && changelogCount < groupedChangelogVersions.length && (
        <div className={styles.changelogSpinner}>
          <Spinner size="tiny" />
        </div>
      )}
    </div>
  );
}
