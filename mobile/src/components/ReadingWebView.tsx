import React, { useEffect, useMemo, useRef } from 'react';
import { StyleSheet, useColorScheme, View } from 'react-native';
import { WebView, WebViewMessageEvent } from 'react-native-webview';
import { palette } from '../lib/theme/typography';
import { BOOKFIN_READER_CSS, BookfinHtmlOptions, renderBookfinPageToHtml } from '../lib/reader/bookfinHtml';
import { parseReaderBridgeMessage } from '../lib/reader/webViewBridge';
import { FeedPageDto } from '../types/api';
import { POLIPHILI_FAMILY, poliphiliIsInstalled } from '../lib/reader/bookfinFont';

interface ReadingWebViewProps {
  page: FeedPageDto;
  options: BookfinHtmlOptions;
  initialScrollOffset?: number;
  onScrollState?: (state: { scrollTop: number; scrollHeight: number; viewportHeight: number; atBottom: boolean }) => void;
  onSwipeLeft?: (atBottom: boolean) => void;
  onSwipeRight?: () => void;
  onFontStatus?: (status: { family: string; requestedMode: 'poliphili' | 'system'; romanLoaded: boolean; italicLoaded: boolean; status: 'loaded' | 'fallback' | 'not_installed' | 'italic_missing' }) => void;
}

const readerShell = `<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1"><style>${BOOKFIN_READER_CSS}</style></head><body><main id="root"></main><script>(function(){var start=null,font={family:${JSON.stringify(POLIPHILI_FAMILY)},installed:${poliphiliIsInstalled ? 'true' : 'false'}};function send(v){window.ReactNativeWebView&&window.ReactNativeWebView.postMessage(JSON.stringify(v));}function reportFont(mode){var finish=function(roman,italic){var st;if(!font.installed&&!roman){st='not_installed';}else if(mode==='system'){st='fallback';}else if(roman&&italic){st='loaded';}else if(roman&&!italic){st='italic_missing';}else{st='fallback';}send({type:'FONT_STATUS',family:font.family,requestedMode:mode,romanLoaded:Boolean(roman),italicLoaded:Boolean(italic),status:st});};if(!document.fonts){finish(false,false);return;}document.fonts.ready.then(function(){var hasItalicFace=false;document.fonts.forEach(function(f){var fam=f.family.replace(/["']/g,'');if(fam===font.family&&f.style==='italic'&&f.status==='loaded'){hasItalicFace=true;}});var r=document.fonts.check('400 19px "'+font.family+'"'),i=hasItalicFace&&document.fonts.check('italic 400 19px "'+font.family+'"');finish(r,i);},function(){finish(false,false);});}function state(){var top=window.scrollY||document.documentElement.scrollTop||0,h=Math.max(document.body.scrollHeight,document.documentElement.scrollHeight),v=window.innerHeight;return {type:'SCROLL_STATE',scrollTop:top,scrollHeight:h,viewportHeight:v,atBottom:top>=h-v-2};}function report(){send(state())}window.BookfinReader={setPage:function(p){document.documentElement.lang=p.language;document.documentElement.style.setProperty('--bf-fg',p.theme.fg);document.documentElement.style.setProperty('--bf-muted',p.theme.muted);document.getElementById('root').innerHTML=p.html;reportFont(p.fontMode||'poliphili');requestAnimationFrame(function(){window.scrollTo(0,p.scrollOffset||0);setTimeout(function(){report();send({type:'READY',pageId:p.pageId})},0)})}};addEventListener('scroll',report,{passive:true});addEventListener('resize',report);addEventListener('touchstart',function(e){var t=e.changedTouches[0];start={x:t.clientX,y:t.clientY}},{passive:true});addEventListener('touchend',function(e){if(!start)return;var t=e.changedTouches[0],dx=t.clientX-start.x,dy=t.clientY-start.y,s=state();if(Math.abs(dx)>50&&Math.abs(dx)>Math.abs(dy)*2.5)send(dx<0?{type:'SWIPE_LEFT',atBottom:s.atBottom}:{type:'SWIPE_RIGHT'});start=null},{passive:true});document.addEventListener('message',function(e){try{window.BookfinReader.setPage(JSON.parse(e.data))}catch(_){}});reportFont('poliphili');})();</script></body></html>`;

/** A single native WebView stays mounted; page changes only inject a new V2-derived article. */
export const ReadingWebView: React.FC<ReadingWebViewProps> = ({ page, options, initialScrollOffset = 0, onScrollState, onSwipeLeft, onSwipeRight, onFontStatus }) => {
  const ref = useRef<WebView>(null);
  const scheme = useColorScheme();
  const theme = scheme === 'dark' ? palette.dark : palette.light;
  const payload = useMemo(() => ({ pageId: page.page_id, language: page.language_tag, html: renderBookfinPageToHtml(page, options), fontMode: options.fontMode ?? 'poliphili', scrollOffset: initialScrollOffset, theme: { fg: theme.textPrimary, muted: theme.textMuted } }), [page, options, initialScrollOffset, theme.textPrimary, theme.textMuted]);

  useEffect(() => {
    const script = `window.BookfinReader&&window.BookfinReader.setPage(${JSON.stringify(payload).replace(/</g, '\\u003c')});true;`;
    ref.current?.injectJavaScript(script);
  }, [payload]);

  const onMessage = (event: WebViewMessageEvent) => {
    const message = parseReaderBridgeMessage(event.nativeEvent.data);
    if (!message) return;
    if (message.type === 'SCROLL_STATE') onScrollState?.(message);
    if (message.type === 'SWIPE_LEFT') onSwipeLeft?.(message.atBottom);
    if (message.type === 'SWIPE_RIGHT') onSwipeRight?.();
    if (message.type === 'FONT_STATUS') onFontStatus?.(message);
  };

  return <View style={styles.container}><WebView ref={ref} source={{ html: readerShell }} onMessage={onMessage} originWhitelist={['*']} javaScriptEnabled domStorageEnabled bounces={false} style={[styles.webView, { backgroundColor: theme.background }]} /></View>;
};

const styles = StyleSheet.create({ container: { flex: 1 }, webView: { flex: 1, backgroundColor: 'transparent' } });
