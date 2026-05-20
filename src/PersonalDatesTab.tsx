import { useCallback, useEffect, useState } from "react";
import {
  Button,
  Card,
  Text,
  Input,
  makeStyles,
  tokens,
  Tooltip,
  Field,
} from "@fluentui/react-components";
import {
  Delete20Regular,
  Edit20Regular,
  Add20Regular,
  Checkmark20Regular,
  Dismiss20Regular,
} from "@fluentui/react-icons";
import { getSettings, saveSettings } from "./api";
import type { PersonalDate, Settings } from "./types";
import { t } from "./i18n";
import { DatePicker } from "@fluentui/react-datepicker-compat";

const useStyles = makeStyles({
  card: {
    marginBottom: tokens.spacingVerticalM,
  },

  addRow: {
    display: "flex",
    gap: tokens.spacingHorizontalS,
    alignItems: "flex-end",
    flexWrap: "wrap",
  },
  addDateField: {
    flex: "1 1 160px",
    minWidth: "160px",
  },
  addLabelField: {
    flex: "2 1 180px",
    minWidth: "120px",
  },
  addButton: {
    flexShrink: 0,
    alignSelf: "flex-end",
    marginBottom: "1px",
  },

  listItem: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    padding: `${tokens.spacingVerticalXS} ${tokens.spacingHorizontalS}`,
    borderRadius: tokens.borderRadiusMedium,
    marginBottom: "2px",
    ":hover": {
      background: tokens.colorNeutralBackground1Hover,
    },
  },
  listItemEditing: {
    flexDirection: "column",
    alignItems: "stretch",
    background: tokens.colorNeutralBackground2,
    padding: tokens.spacingVerticalS,
    gap: tokens.spacingVerticalS,
  },

  listLeft: {
    display: "flex",
    alignItems: "center",
    flex: 1,
    minWidth: 0,
    gap: tokens.spacingHorizontalS,
  },
  listDate: {
    fontVariantNumeric: "tabular-nums",
    color: tokens.colorNeutralForeground2,
    flexShrink: 0,
    fontSize: tokens.fontSizeBase200,
  },
  listLabel: {
    flex: 1,
    minWidth: 0,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
  listYear: {
    fontSize: tokens.fontSizeBase100,
    color: tokens.colorNeutralForeground3,
  },
  actions: {
    display: "flex",
    flexShrink: 0,
    marginLeft: tokens.spacingHorizontalS,
  },

  editRow: {
    display: "flex",
    gap: tokens.spacingHorizontalS,
    alignItems: "flex-end",
    flexWrap: "wrap",
  },
  editDateField: {
    flex: "1 1 160px",
    minWidth: "160px",
  },
  editLabelField: {
    flex: "2 1 180px",
    minWidth: "120px",
  },
  editActions: {
    display: "flex",
    gap: tokens.spacingHorizontalXS,
    justifyContent: "flex-end",
  },
});

export default function PersonalDatesTab() {
  const styles = useStyles();
  const [dates, setDates] = useState<PersonalDate[]>([]);
  const [loading, setLoading] = useState(true);
  const [newDate, setNewDate] = useState<Date | null>(null);
  const [newLabel, setNewLabel] = useState("");
  const [editingIndex, setEditingIndex] = useState<number | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const s: Settings = await getSettings();
        setDates(s.personalDates ?? []);
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const persist = useCallback(async (next: PersonalDate[]) => {
    try {
      const s: Settings = await getSettings();
      await saveSettings({ ...s, personalDates: next });
      setDates(next);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const addDate = useCallback(async () => {
    if (!newDate || !newLabel.trim()) return;
    await persist([
      ...dates,
      {
        month: newDate.getMonth() + 1,
        day: newDate.getDate(),
        year: newDate.getFullYear(),
        label: newLabel.trim(),
      },
    ]);
    setNewDate(null);
    setNewLabel("");
  }, [dates, newDate, newLabel, persist]);

  const removeDate = useCallback(
    async (idx: number) => {
      await persist(dates.filter((_, i) => i !== idx));
    },
    [dates, persist]
  );

  const saveEdit = async (idx: number, updated: PersonalDate) => {
    await persist(dates.map((d, i) => (i === idx ? updated : d)));
    setEditingIndex(null);
  };

  if (loading) return <Text>{t("loading")}</Text>;

  return (
    <>
      {/* Форма додавання */}
      <Card className={styles.card}>
        <Text weight="semibold" size={200} block>
          {t("personal_dates.title")}
        </Text>

        <div className={styles.addRow}>
          <div className={styles.addDateField}>
            <Field label={t("personal_dates.date")}>
              <DatePicker
                placeholder={t("personal_dates.date")}
                value={newDate}
                onSelectDate={(d) => setNewDate(d ?? null)}
              />
            </Field>
          </div>

          <div className={styles.addLabelField}>
            <Field label={t("personal_dates.label")}>
              <Input
                placeholder={t("personal_dates.label")}
                value={newLabel}
                onChange={(_, d) => setNewLabel(d.value)}
                onKeyDown={(e) => e.key === "Enter" && addDate()}
              />
            </Field>
          </div>

          <Button
            className={styles.addButton}
            appearance="primary"
            icon={<Add20Regular />}
            onClick={addDate}
            disabled={!newDate || !newLabel.trim()}
          >
            {t("buttons.add")}
          </Button>
        </div>
      </Card>

      {/* Список */}
      <Card className={styles.card}>
        <Text weight="semibold" size={200} block>
          {t("personal_dates.list_title")}
        </Text>

        {dates.length === 0 && (
          <Text size={200} style={{ color: tokens.colorNeutralForeground3 }}>
            {t("personal_dates.empty")}
          </Text>
        )}

        {dates.map((d, idx) =>
          editingIndex === idx ? (
            <div
              key={idx}
              className={`${styles.listItem} ${styles.listItemEditing}`}
            >
              <EditForm
                initial={d}
                onCancel={() => setEditingIndex(null)}
                onSave={(v) => saveEdit(idx, v)}
              />
            </div>
          ) : (
            <div key={idx} className={styles.listItem}>
              <div className={styles.listLeft}>
                <span className={styles.listDate}>
                  {String(d.month).padStart(2, "0")}/{String(d.day).padStart(2, "0")}
                </span>
                <div style={{ minWidth: 0 }}>
                  <div className={styles.listLabel}>{d.label}</div>
                  {d.year && <div className={styles.listYear}>{d.year}</div>}
                </div>
              </div>

              <div className={styles.actions}>
                <Tooltip
                  content={t("buttons.edit")}
                  relationship="label"
                >
                  <Button
                    appearance="transparent"
                    icon={<Edit20Regular />}
                    onClick={() => setEditingIndex(idx)}
                  />
                </Tooltip>
                <Tooltip
                  content={t("buttons.delete")}
                  relationship="label"
                >
                  <Button
                    appearance="transparent"
                    icon={<Delete20Regular />}
                    onClick={() => removeDate(idx)}
                  />
                </Tooltip>
              </div>
            </div>
          )
        )}
      </Card>
    </>
  );
}

function EditForm({
  initial,
  onSave,
  onCancel,
}: {
  initial: PersonalDate;
  onSave: (v: PersonalDate) => void;
  onCancel: () => void;
}) {
  const styles = useStyles();
  const [date, setDate] = useState<Date | null>(() => {
    const year = initial.year ?? new Date().getFullYear();
    return new Date(year, initial.month - 1, initial.day);
  });
  const [label, setLabel] = useState(initial.label);

  const handleSave = () => {
    if (!date || !label.trim()) return;
    onSave({
      month: date.getMonth() + 1,
      day: date.getDate(),
      year: date.getFullYear(),
      label: label.trim(),
    });
  };

  return (
    <>
      <div className={styles.editRow}>
        <div className={styles.editDateField}>
          <Field label={t("personal_dates.date")}>
            <DatePicker
              value={date}
              onSelectDate={(d) => setDate(d ?? null)}
            />
          </Field>
        </div>
        <div className={styles.editLabelField}>
          <Field label={t("personal_dates.label")}>
            <Input
              value={label}
              onChange={(_, d) => setLabel(d.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSave()}
            />
          </Field>
        </div>
      </div>

      <div className={styles.editActions}>
        <Button
          appearance="subtle"
          icon={<Dismiss20Regular />}
          onClick={onCancel}
        >
          {t("buttons.cancel")}
        </Button>
        <Button
          appearance="primary"
          icon={<Checkmark20Regular />}
          onClick={handleSave}
          disabled={!date || !label.trim()}
        >
          {t("buttons.save")}
        </Button>
      </div>
    </>
  );
}
