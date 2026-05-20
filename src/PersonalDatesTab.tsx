import { useCallback, useEffect, useState } from "react";
import {
  Button,
  Card,
  Text,
  Input,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import { Delete20Regular, Edit20Regular, Add20Regular } from "@fluentui/react-icons";
import { getSettings, saveSettings } from "./api";
import type { PersonalDate, Settings } from "./types";
import { t } from "./i18n";
import { Field } from "@fluentui/react-components";
import { DatePicker } from "@fluentui/react-datepicker-compat";

const useStyles = makeStyles({
  card: {
    marginBottom: tokens.spacingVerticalM,
    padding: tokens.spacingVerticalS,
  },
  row: {
    display: "flex",
    gap: tokens.spacingHorizontalS,
    alignItems: "center",
    marginBottom: tokens.spacingVerticalS,
  },
  listItem: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalS}`,
    borderRadius: tokens.borderRadiusMedium,
    background: tokens.colorNeutralBackground1,
    marginBottom: tokens.spacingVerticalXS,
  },
  listLabel: {
    flex: 1,
    marginLeft: tokens.spacingHorizontalS,
  },
});

export default function PersonalDatesTab() {
  const styles = useStyles();
  const [dates, setDates] = useState<PersonalDate[]>([]);
  const [loading, setLoading] = useState(true);

  const [newDate, setNewDate] = useState<Date | null>(null);
  const [newLabel, setNewLabel] = useState<string>("");

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
      const updated: Settings = { ...s, personalDates: next };
      await saveSettings(updated);
      setDates(next);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const addDate = useCallback(async () => {
    if (!newDate || !newLabel.trim()) return;
    const month = newDate.getMonth() + 1;
    const day = newDate.getDate();
    const year = newDate.getFullYear();
    const next: PersonalDate[] = [
      ...dates,
      { month, day, label: newLabel.trim(), year },
    ];
    await persist(next);
    setNewDate(null);
    setNewLabel("");
  }, [dates, newDate, newLabel, persist]);

  const removeDate = useCallback(async (idx: number) => {
    const next = dates.filter((_, i) => i !== idx);
    await persist(next);
  }, [dates, persist]);

  const startEdit = (idx: number) => setEditingIndex(idx);

  const saveEdit = async (idx: number, updated: PersonalDate) => {
    const next = dates.map((d, i) => (i === idx ? updated : d));
    await persist(next);
    setEditingIndex(null);
  };

  if (loading) return <div>{t("loading") ?? "Loading..."}</div>;

  return (
    <>
      <Card className={styles.card}>
        <Text weight="semibold" size={200} block>
          {t("personal_dates.title") ?? "Personal Dates"}
        </Text>

        <div className={styles.row}>
          <Field label={t("personal_dates.title") ?? "Date"}>
            <DatePicker
              placeholder={t("personal_dates.month") ?? "Select a date..."}
              value={newDate}
              onSelectDate={(d) => setNewDate(d ?? null)}
              className={styles.row}
            />
          </Field>

          <Input
            placeholder={t("personal_dates.label") ?? "Label"}
            value={newLabel}
            onChange={(_, d) => setNewLabel(d.value)}
            style={{ flex: 1 }}
          />

          <Button appearance="primary" icon={<Add20Regular />} onClick={addDate}>
            {t("buttons.add") ?? "Add"}
          </Button>
        </div>
      </Card>

      <Card className={styles.card}>
        <Text weight="semibold" size={200} block>
          {t("personal_dates.list_title") ?? "Saved dates"}
        </Text>
        <div style={{ marginTop: tokens.spacingVerticalS }}>
          {dates.length === 0 && <div>{t("personal_dates.empty") ?? "No personal dates."}</div>}
          {dates.map((d, idx) => (
            <div key={idx} className={styles.listItem}>
              <div style={{ display: "flex", alignItems: "center" }}>
                <div>
                  <strong>{String(d.month).padStart(2, "0")}/{String(d.day).padStart(2, "0")}</strong>
                </div>
                <div className={styles.listLabel}>
                  <div>{d.label}</div>
                  <div style={{ fontSize: 12, color: tokens.colorNeutralForeground3 }}>
                    {d.year ? d.year : ""}
                  </div>
                </div>
              </div>

              <div style={{ display: "flex", gap: tokens.spacingHorizontalS }}>
                <Button appearance="transparent" icon={<Edit20Regular />} onClick={() => startEdit(idx)} />
                <Button appearance="transparent" icon={<Delete20Regular />} onClick={() => removeDate(idx)} />
              </div>
            </div>
          ))}
        </div>
      </Card>

      {editingIndex !== null && (
        <Card className={styles.card}>
          <Text weight="semibold" size={200} block>
            {t("personal_dates.edit_title") ?? "Edit date"}
          </Text>
          <EditForm
            initial={dates[editingIndex]}
            onCancel={() => setEditingIndex(null)}
            onSave={(v) => saveEdit(editingIndex, v)}
          />
        </Card>
      )}
    </>
  );
}

function EditForm({ initial, onSave, onCancel }: { initial: PersonalDate; onSave: (v: PersonalDate) => void; onCancel: () => void; }) {
  const [month, setMonth] = useState(String(initial.month));
  const [day, setDay] = useState(String(initial.day));
  const [year, setYear] = useState(initial.year ? String(initial.year) : "");
  const [label, setLabel] = useState(initial.label);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacingVerticalS }}>
      <div style={{ display: "flex", gap: tokens.spacingHorizontalS }}>
        <Input value={month} onChange={(_, d) => setMonth(d.value)} type="number" style={{ width: 80 }} />
        <Input value={day} onChange={(_, d) => setDay(d.value)} type="number" style={{ width: 80 }} />
        <Input value={year} onChange={(_, d) => setYear(d.value)} type="number" style={{ width: 120 }} />
        <Input value={label} onChange={(_, d) => setLabel(d.value)} style={{ flex: 1 }} />
      </div>
      <div style={{ display: "flex", gap: tokens.spacingHorizontalS }}>
        <Button appearance="primary" onClick={() => onSave({ month: Number(month), day: Number(day), year: year ? Number(year) : null, label: label.trim() })}>
          {t("buttons.save") ?? "Save"}
        </Button>
        <Button appearance="subtle" onClick={onCancel}>{t("buttons.cancel") ?? "Cancel"}</Button>
      </div>
    </div>
  );
}
