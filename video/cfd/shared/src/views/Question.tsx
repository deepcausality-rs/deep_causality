/**
 * The questions a cut answers: the agenda that lists them where the story first poses them, and
 * one question set large where the story returns to it.
 */
import { color, font } from '../tokens';

/** The questions, numbered; `titleOpacity` fades the heading, `lineOpacity(i)` each question. */
export const Agenda: React.FC<{ questions: string[]; x: number; y: number; width?: number; titleOpacity: number; lineOpacity: (i: number) => number }> = ({
  questions,
  x,
  y,
  width = 800,
  titleOpacity,
  lineOpacity,
}) => (
  <div style={{ position: 'absolute', left: x, top: y, width }}>
    <div style={{ fontFamily: font.mono, fontSize: 22, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent, opacity: titleOpacity }}>
      {questions.length === 2 ? 'Two questions' : `${questions.length} questions`}
    </div>
    {questions.map((q, i) => (
      <div key={q} style={{ marginTop: 18, display: 'flex', gap: 22, fontFamily: font.sans, fontSize: 40, fontWeight: 500, color: color.fg0, opacity: lineOpacity(i) }}>
        <span style={{ fontFamily: font.mono, color: color.accent }}>{i + 1}</span>
        {q}
      </div>
    ))}
  </div>
);

export const QuestionTitle: React.FC<{ n: number; text: string; x: number; y: number; opacity: number }> = ({ n, text, x, y, opacity }) => (
  <div style={{ position: 'absolute', left: x, top: y, opacity }}>
    <div style={{ fontFamily: font.mono, fontSize: 22, letterSpacing: '0.16em', textTransform: 'uppercase', color: color.accent }}>Question {n}</div>
    <div style={{ marginTop: 10, fontFamily: font.sans, fontSize: 60, fontWeight: 500, letterSpacing: '-0.01em', color: color.fg0 }}>{text}</div>
  </div>
);
