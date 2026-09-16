import { BookfinContentBlock, BookfinTextSpan, FeedPageDto } from '../../types/api';
import Hypher from 'hypher';
import frenchPatterns from 'hyphenation.fr';
import englishPatterns from 'hyphenation.en-us';
import spanishPatterns from 'hyphenation.es';
import { BOOKFIN_FONT_FALLBACK, BOOKFIN_FONT_FAMILY_STACK, BOOKFIN_POLIPHILI_FONT_FACE_CSS, POLIPHILI_FAMILY } from './bookfinFont';
import type { ReaderFontMode } from './bookfinFont';

export type ReaderParagraphStyle = 'book' | 'balanced' | 'screen';

export interface BookfinHtmlOptions {
  paragraphStyle?: ReaderParagraphStyle;
  justify?: boolean;
  softHyphens?: boolean;
  showBlockTypes?: boolean;
  frenchMin?: 3 | 4;
  fontMode?: ReaderFontMode;
}

const LEFT_MIN = 3;
const RIGHT_MIN = 3;
const HYPHENATORS: Record<string, Hypher> = {
  fr: new Hypher(frenchPatterns),
  en: new Hypher(englishPatterns),
  es: new Hypher(spanishPatterns),
};

export function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

/** TeX/Liang language patterns, applied only to controlled HTML text. Existing
 * punctuation, apostrophes, numbers, URLs and hyphenated tokens are excluded. */
export function insertSoftHyphens(text: string, language: string): string {
  return insertSoftHyphensWithMin(text, language);
}

export function insertSoftHyphensWithMin(text: string, language: string, frenchMin: 3 | 4 = 4): string {
  const hypher = HYPHENATORS[language.toLowerCase()];
  if (!hypher) return text;
  const leftMin = language.toLowerCase() === 'fr' ? frenchMin : LEFT_MIN;
  const rightMin = language.toLowerCase() === 'fr' ? frenchMin : RIGHT_MIN;
  const hyphenateSegment = (segment: string) => segment.replace(/[\p{L}]+/gu, (word) => {
    if (word.length < leftMin + rightMin + 1) return word;
    const parts = hypher.hyphenate(word);
    const breaks = new Set<number>();
    let offset = 0;
    let previousBreak = 0;
    for (let index = 0; index < parts.length - 1; index += 1) {
      offset += parts[index].length;
      if (offset - previousBreak >= leftMin && word.length - offset >= rightMin) {
        breaks.add(offset);
        previousBreak = offset;
      }
    }
    return Array.from(word).map((letter, index) => `${letter}${breaks.has(index + 1) ? '\u00ad' : ''}`).join('');
  });
  return text.split(/(https?:\/\/[^\s]+)/giu).map((segment) => /^https?:\/\//i.test(segment) ? segment : hyphenateSegment(segment)).join('');
}

export function stripSoftHyphens(text: string): string { return text.replace(/\u00ad/g, ''); }
export function countSoftHyphens(text: string): number { return (text.match(/\u00ad/g) ?? []).length; }

function renderSpan(span: BookfinTextSpan, language: string, softHyphens: boolean, preserveLines = false, frenchMin: 3 | 4 = 4): string {
  let content = softHyphens ? insertSoftHyphensWithMin(span.text, language, frenchMin) : span.text;
  content = escapeHtml(content);
  if (preserveLines) content = content.replace(/\r?\n/g, '<br>');
  if (span.small_caps) content = `<span class="bf-small-caps">${content}</span>`;
  if (span.bold) content = `<strong>${content}</strong>`;
  if (span.italic) content = `<em>${content}</em>`;
  return content;
}

function renderSpans(block: BookfinContentBlock, language: string, softHyphens: boolean, preserveLines = false, frenchMin: 3 | 4 = 4): string {
  return (block.spans ?? []).map((span) => renderSpan(span, language, softHyphens, preserveLines, frenchMin)).join('');
}

export function renderBookfinBlocksToHtml(blocks: BookfinContentBlock[], language: string, options: BookfinHtmlOptions = {}): string {
  // Patterns stay opt-in until their new device validation is complete.
  const softHyphens = options.softHyphens ?? false;
  return blocks.map((block) => {
    const diagnostic = options.showBlockTypes ? `<span class="bf-diagnostic">${escapeHtml(block.type)}</span>` : '';
    if (block.type === 'scene_break') return `<div class="bf-scene-break" role="separator">${diagnostic}<span>* * *</span></div>`;
    if (block.type === 'heading') return `<h${Math.min(3, Math.max(2, block.level ?? 2))}>${diagnostic}${renderSpans(block, language, softHyphens, false, options.frenchMin)}</h${Math.min(3, Math.max(2, block.level ?? 2))}>`;
    if (block.type === 'blockquote') return `<blockquote>${diagnostic}${renderSpans(block, language, softHyphens, false, options.frenchMin)}</blockquote>`;
    if (block.type === 'verse') return `<div class="bf-verse">${diagnostic}${renderSpans(block, language, softHyphens, true, options.frenchMin)}</div>`;
    return `<p>${diagnostic}${renderSpans(block, language, softHyphens, false, options.frenchMin)}</p>`;
  }).join('');
}

export function renderBookfinPageToHtml(page: FeedPageDto, options: BookfinHtmlOptions = {}): string {
  const language = page.language_tag || 'en';
  const content = page.blocks?.length ? renderBookfinBlocksToHtml(page.blocks, language, options) : `<p>${escapeHtml(page.text || page.content || '')}</p>`;
  const style = options.paragraphStyle ?? 'balanced';
  const fontMode = options.fontMode ?? 'poliphili';
  return `<article class="bf-reader bf-font-${fontMode} bf-${style}-style ${options.softHyphens ? 'bf-hyphenation-patterns' : 'bf-hyphenation-off'} ${options.justify === false ? 'bf-no-justify' : ''}" lang="${escapeHtml(language)}">${content}</article>`;
}

export const BOOKFIN_READER_CSS = `
${BOOKFIN_POLIPHILI_FONT_FACE_CSS}*{box-sizing:border-box}html,body{margin:0;min-height:100%;background:transparent;color:var(--bf-fg);font-family:${BOOKFIN_FONT_FAMILY_STACK};font-synthesis:none;-webkit-font-synthesis:none}body{padding:16px 24px 32px;font-size:19px;line-height:1.62;letter-spacing:.01em}article{max-width:640px;margin:0 auto}.bf-font-system{font-family:${BOOKFIN_FONT_FALLBACK}}p{margin:0 0 .25em;text-align:justify;overflow-wrap:break-word;hyphens:none;-webkit-hyphens:none}.bf-hyphenation-patterns p{hyphens:manual;-webkit-hyphens:manual}.bf-hyphenation-off p{hyphens:none;-webkit-hyphens:none}.bf-book-style p+p{text-indent:1.15em}.bf-balanced-style p{margin:0 0 .12em}.bf-balanced-style p+p{text-indent:.9em}.bf-screen-style p{margin:0 0 .75em;text-indent:0}.bf-no-justify p{text-align:start}h2,h3{margin:1.25em 0 .8em;text-align:center;font-size:1.08em;line-height:1.35;font-weight:600;letter-spacing:.03em}.bf-scene-break{margin:1.4em 0;text-align:center;letter-spacing:.25em}.bf-scene-break .bf-diagnostic{display:block;letter-spacing:normal}blockquote{margin:.8em 1em;padding:0;font-style:italic;text-align:justify;hyphens:none;-webkit-hyphens:none}.bf-verse{margin:.8em 0;white-space:normal;text-align:start;hyphens:none;-webkit-hyphens:none;overflow-wrap:break-word}.bf-small-caps{font-variant:small-caps}.bf-diagnostic{font-family:system-ui,sans-serif;font-size:.55em;letter-spacing:.08em;color:var(--bf-muted);margin-right:.5em;text-indent:0}
`;
