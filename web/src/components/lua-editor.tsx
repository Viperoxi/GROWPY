import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
  drawSelection,
  placeholder as cmPlaceholder,
} from "@codemirror/view";
import { EditorState, Prec } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  StreamLanguage,
  syntaxHighlighting,
  HighlightStyle,
  indentOnInput,
  bracketMatching,
} from "@codemirror/language";
import { lua } from "@codemirror/legacy-modes/mode/lua";
import { tags as t } from "@lezer/highlight";
import { cn } from "@/lib/utils";

const sublimeHighlight = HighlightStyle.define([
  { tag: t.keyword, color: "#F92672" },
  { tag: t.controlKeyword, color: "#F92672" },
  { tag: t.operatorKeyword, color: "#F92672" },
  { tag: t.definitionKeyword, color: "#F92672" },
  { tag: t.moduleKeyword, color: "#F92672" },
  { tag: t.string, color: "#E6DB74" },
  { tag: t.character, color: "#E6DB74" },
  { tag: t.comment, color: "#75715E", fontStyle: "italic" },
  { tag: t.lineComment, color: "#75715E", fontStyle: "italic" },
  { tag: t.blockComment, color: "#75715E", fontStyle: "italic" },
  { tag: t.number, color: "#AE81FF" },
  { tag: t.bool, color: "#AE81FF" },
  { tag: t.null, color: "#AE81FF" },
  { tag: t.atom, color: "#AE81FF" },
  { tag: t.function(t.variableName), color: "#A6E22E" },
  { tag: t.function(t.propertyName), color: "#A6E22E" },
  { tag: t.definition(t.variableName), color: "#FD971F" },
  { tag: t.variableName, color: "#F8F8F2" },
  { tag: t.propertyName, color: "#A6E22E" },
  { tag: t.operator, color: "#F92672" },
  { tag: t.punctuation, color: "#F8F8F2" },
  { tag: t.paren, color: "#F8F8F2" },
  { tag: t.bracket, color: "#F8F8F2" },
  { tag: t.special(t.variableName), color: "#66D9EF" },
]);

const sublimeTheme = EditorView.theme(
  {
    "&": {
      backgroundColor: "#272822",
      color: "#F8F8F2",
      height: "100%",
      fontSize: "13px",
    },
    ".cm-scroller": {
      overflow: "auto",
      fontFamily:
        'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace',
    },
    ".cm-content": {
      caretColor: "#F8F8F0",
      padding: "10px 0",
    },
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "#F8F8F0" },
    ".cm-gutters": {
      backgroundColor: "#272822",
      color: "#8F908A",
      border: "none",
    },
    ".cm-activeLine": { backgroundColor: "#3E3D32" },
    ".cm-activeLineGutter": { backgroundColor: "#3E3D32" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      backgroundColor: "#49483E",
    },
    ".cm-placeholder": { color: "#75715E" },
  },
  { dark: true },
);

export type LuaEditorHandle = {
  focus: () => void;
  getCursor: () => number;
  replaceRange: (from: number, to: number, text: string, cursor: number) => void;
};

type Props = {
  value: string;
  onChange: (value: string) => void;
  className?: string;
  placeholder?: string;
  onCursor?: (value: string, pos: number) => void;
  onSpecialKey?: (key: string) => boolean;
};

export const LuaEditor = forwardRef<LuaEditorHandle, Props>(function LuaEditor(
  { value, onChange, className, placeholder, onCursor, onSpecialKey },
  ref,
) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const valueRef = useRef(value);
  const onChangeRef = useRef(onChange);
  const onCursorRef = useRef(onCursor);
  const onSpecialKeyRef = useRef(onSpecialKey);

  valueRef.current = value;
  onChangeRef.current = onChange;
  onCursorRef.current = onCursor;
  onSpecialKeyRef.current = onSpecialKey;

  useImperativeHandle(ref, () => ({
    focus: () => viewRef.current?.focus(),
    getCursor: () => viewRef.current?.state.selection.main.head ?? 0,
    replaceRange: (from, to, text, cursor) => {
      const view = viewRef.current;
      if (!view) return;
      view.dispatch({
        changes: { from, to, insert: text },
        selection: { anchor: cursor },
      });
      view.focus();
    },
  }));

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: valueRef.current,
        extensions: [
          lineNumbers(),
          highlightActiveLine(),
          highlightActiveLineGutter(),
          drawSelection(),
          history(),
          indentOnInput(),
          bracketMatching(),
          StreamLanguage.define(lua),
          syntaxHighlighting(sublimeHighlight),
          sublimeTheme,
          EditorView.lineWrapping,
          cmPlaceholder(placeholder ?? ""),
          Prec.highest(
            keymap.of(
              ["ArrowDown", "ArrowUp", "Enter", "Tab", "Escape"].map((key) => ({
                key,
                run: () => onSpecialKeyRef.current?.(key) ?? false,
              })),
            ),
          ),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              const next = update.state.doc.toString();
              valueRef.current = next;
              onChangeRef.current(next);
            }
            if (update.docChanged || update.selectionSet) {
              onCursorRef.current?.(
                update.state.doc.toString(),
                update.state.selection.main.head,
              );
            }
          }),
        ],
      }),
    });
    viewRef.current = view;
    return () => {
      view.destroy();
      viewRef.current = null;
    };
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current === value) return;
    view.dispatch({
      changes: { from: 0, to: current.length, insert: value },
    });
  }, [value]);

  return (
    <div
      ref={hostRef}
      className={cn(
        "min-h-0 flex-1 overflow-hidden rounded-lg border border-border [&_.cm-editor]:h-full [&_.cm-editor]:outline-none",
        className,
      )}
    />
  );
});
