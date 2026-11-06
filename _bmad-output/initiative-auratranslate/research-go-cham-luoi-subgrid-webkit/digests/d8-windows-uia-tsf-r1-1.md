# D8 - Windows UIA va TSF: chi phi moi phim cua Chromium (r1-1)

Ngay truy cap moi nguon: 2026-11-06. Ngan sach da dung: 15 lan goi cong cu, khoang 6 nguon doc day du (cac trang con lai chi thay qua ket qua tim kiem). Moi claim ben duoi chi dua tren ket qua tim duoc o luot nay.

## Claims

1. Cay accessibility cua Chromium khong bat mac dinh; bo "Accessibility features in Chrome are off by default and enabled automatically on-demand". Nguon: https://chromium.googlesource.com/chromium/src/+/main/docs/accessibility/overview.md . Publisher: Chromium project. Ngay: khong ghi (tai lieu song, nhanh main). Confidence: high. Class: spec.

2. Tren Windows, `BrowserAccessibilityStateImpl` (ban Win) chay `DiscoverAssistiveTech` tren thread pool, kiem: Sticky Keys (`SPI_GETSTICKYKEYS`, `SKF_STICKYKEYSON`), Narrator (registry `NoRoam`, gia tri `RunningState`), Windows Magnifier (registry `ScreenMagnifier`, `RunningState`), va cac module nap vao tien trinh (`jhook.dll`, `nvdahelperremote.dll`, `vbufbackend_gecko_ia2.dll`, `dolwinhk.dll`, `zslhook.dll`, `outhelper.dll`, va `uiautomation.dll`/`uiautomationcore.dll`). Nguon: https://chromium.googlesource.com/chromium/src/+/main/content/browser/accessibility/browser_accessibility_state_impl_win.cc . Publisher: Chromium project. Ngay: khong ghi. Confidence: medium (doc qua WebFetch tom tat, khong doc nguyen van tung dong). Class: source-code.

3. Cung file: `uiautomation.dll` duoc gan nhan "API support library, not an actual AT" va khong coi la screen reader. Khi tim ra screen reader, no dat `ui::kAXModeComplete | ui::AXMode::kScreenReader | ui::AXMode::kFromPlatform`. Nghia la Narrator dang CHAY (RunningState) hoac Magnifier dang chay hoac Sticky Keys bat se keo che do complete; chi cai san Narrator thi khong. Confidence: medium (nhu tren). Class: source-code.

4. File Win KHONG nhac touch keyboard, TSF hay text services nhu mot tac nhan kich hoat (tom tat WebFetch noi "no trigger, exclusion, or comment mentions them"). Day la bang chung vang mat trong file nay, khong chung minh rang khong co duong khac (vi du `OnPropertiesUsedInWebContent`, `OnMinimalPropertiesUsed`, `OnPropertiesUsedInBrowserUI`). Nguon: nhu claim 2. Confidence: medium. Class: source-code.

5. Ban lop co so: `OnPropertiesUsedInWebContent()` bat `ui::kAXModeBasic`; `OnMinimalPropertiesUsed()` va `OnPropertiesUsedInBrowserUI()` chi bat `kNativeAPIs`; `kAXModeComplete` khong do tin hieu nen tang dat o file co so ma o che do ep (forced) va `OnActionFromAssistiveTech()`. Ngoai ra co "screen reader honeypot" (chi Windows) goi `OnPropertiesUsedInWebContent` (tu snippet tim kiem ax_platform.h, khong doc truc tiep). Nguon: https://chromium.googlesource.com/chromium/src/+/main/content/browser/accessibility/browser_accessibility_state_impl.cc . Confidence: medium. Class: source-code.

6. Co nghien cuu rieng: 5-10% nguoi dung co accessibility bat, thuong do trinh quan ly mat khau va phan mem diet virus dung API accessibility cua nen tang; Chrome dang chay thu nghiem "Auto Disable Accessibility" va noi tac dong "quite high". Nguon: https://developer.chrome.com/blog/chromium-accessibility-performance . Publisher: Chrome for Developers. Ngay: khong ghi trong ban doc. Confidence: medium-high. Class: performance-number/pattern. Y nghia: co tac nhan khong phai screen reader co the bat cay tren may thuong; can doc chinh xac co che.

7. Chi phi moi cap nhat: khi mot node doi, node bi danh dau dirty va TAT CA thuoc tinh cua no (ca phan khong doi) duoc serialize gui sang browser; bai viet dung cho viec dirty-marking nhanh nhu luc cuon (hon 20 lan/giay). Day la chi phi theo node bi doi, khong theo ca cay, o muc mo ta; bai KHONG noi gi ve go phim, contenteditable hay ruby. Nguon: https://developer.chrome.com/blog/chromium-accessibility-performance . Confidence: medium. Class: pattern. Overview.md cung noi gui lai ca cay moi lan doi la "terribly inefficient" nen dung cap nhat tang dan.

8. Tu Chrome 138 (bai ngay 2025-08-14) cac trinh duyet Chromium tren Windows bat UIA native mac dinh; policy `UiAutomationProviderEnabled` quay lai hanh vi cu, ho tro den Chrome 146 (nghia la o 156 policy nay co the da bi go - chua kiem). Khong co co dong lenh nhac trong bai. Nguon: https://developer.chrome.com/blog/windows-uia-support-update . Publisher: Chrome for Developers. Confidence: medium-high. Class: version. Luu y: UIA provider bat khac voi viec cay duoc XAY; cay chi xay khi co che do (xem claim 2-3).

9. WebView2: tai lieu Microsoft (cap nhat 2026-10-13) liet ke `force-renderer-accessibility`: "Whether to force renderer accessibility to be on, instead of enabling renderer accessibility on-demand when a screen reader is detected. The `disable-renderer-accessibility` flag overrides this flag, if ... present". Tai lieu nay xac nhan co co dong "disable-renderer-accessibility" ton tai trong ngu canh WebView2 va renderer accessibility mac dinh la on-demand; nhung bang chinh KHONG co hang rieng cho `disable-renderer-accessibility` va canh bao co co the bi go/doi bat cu luc nao, khong dung cho production. Cach truyen: bien moi truong `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` hoac `CoreWebView2EnvironmentOptions.AdditionalBrowserArguments`. Nguon: https://learn.microsoft.com/microsoft-edge/webview2/concepts/webview-features-flags . Publisher: Microsoft. Confidence: high cho "tai lieu noi nhu vay"; low cho "co hieu luc o runtime 156.0.4314.58" (chua do). Class: spec/version.

10. `--disable-renderer-accessibility` van duoc dung trong ma nguon browser_accessibility_state_impl.cc: "locks changes" (khoa khong cho doi che do). Nguon: claim 5. Confidence: medium. Class: source-code. Day la ban main, khong phai tag 156 cu the.

11. Ve go phim va layout (khong phai accessibility): issue Chromium https://issues.chromium.org/issues/40688351 "Continues typing in contenteditable is slow in certain cases" (ket qua tim kiem, khong doc day du; trang thai hien tai chua xac dinh) co binh luan rang dat div contenteditable `position:absolute` bo lag, gan cho tranh reflow. Ngoai ra thread ProseMirror (https://discuss.prosemirror.net/t/performance-issues-with-prosemirror-and-chrome/2498) bao Chrome lag voi contenteditable lon, tang khi tat spellcheck, va giau phan lon tai lieu bang display:none dua go phim ve muc chap nhan duoc. Mot binh luan issue Chromium 41237496 (qua snippet, khong mo duoc) noi phan lon thoi gian la tao PlainTextRange nhieu lan moi input va moi ky tu can layout sach. Confidence: low-medium (snippet thu cap). Class: bug-status (trang thai chua kiem).

12. TSF: chi tim duoc bang chung gian tiep tu Firefox (https://bugzilla.mozilla.org/show_bug.cgi?id=1838355): TSF goi lap `GetSelection()`, `GetText()`, `GetTextExt()` va moi truy van dung phang noi dung ca editor khi vung chon lon. Day KHONG phai Chromium. Confidence: low cho Chromium. Class: pattern.

13. Ruby: vai tro `kRuby` va `kRubyAnnotation` ton tai trong ax_enums.mojom (ban Chromium 88) va Blink co `RubyRole` tu 2014 (https://codereview.chromium.org/710663002/). Khong tim thay anh xa hien hanh `<rt>` -> role nao, hay co chi phi dac biet. Confidence: low-medium. Class: source-code.

## Leads dang theo

- Doc nguyen van `browser_accessibility_state_impl_win.cc` va `ax_platform.h` / `ui/accessibility/platform/` de xac nhan duong "screen reader honeypot" va `UiaClientInfoSourceMonitor` (ai bat `kNativeAPIs`/`kWebContents`), dac biet xem touch keyboard/TSF co di qua `OnPropertiesUsedInWebContent` khong.
- Do truc tiep tren may Windows: chay WebView2 voi `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--disable-renderer-accessibility` va so sanh; goi `chrome://accessibility` (edge://accessibility) de xem mode hien tai (day la phep thu quyet dinh, re).
- Kiem tra `ax_tree_serializer` va `AXObjectCacheImpl` ve text-changed trong contenteditable (co dirty ca cay con cua region editable khong).
- Mo issue 40688351 va 41237496 de lay trang thai hien tai va ngay.
- Kiem tra TSF: `ui/base/ime/win/tsf_text_store.cc` (`GetTextExt`, `GetACPFromPoint`, `OnTextChange`) va `TSFBridge`; xem co `RequestTextInputStateUpdate` / ep layout moi thay doi khong.
- Neu `UiAutomationProviderEnabled` bi go o 156, UIA native la mac dinh khong tat duoc bang policy.

## Khong tim thay (vang mat bang chung)

- Khong tim thay nguon nao noi touch keyboard, text prediction, suggested actions, Windows Input Experience hay TSF bat cay accessibility vo dieu kien. Cung khong co nguon phu nhan. Ket qua: chua xac lap.
- Khong tim thay so do hay bug Chromium noi cu the rang accessibility lam go phim cham trong contenteditable lon tren Windows. Khong co bang chung chi phi theo tong kich thuoc DOM moi phim.
- Khong tim thay ca bao cao nao ve `--disable-renderer-accessibility` sua do tre go phim (Chromium hoac WebView2). Cung khong co bao cao no khong hoat dong trong WebView2.
- Khong tim thay so do ve chi phi TSF moi phim trong Chromium (co khi ep layout qua caret rect/text extent hay khong) hay bug Chromium ve do tre input do TSF voi ban phim US khong IME.
- Khong tim thay tai lieu ve chi phi dac biet cua `<ruby>`/`<rt>` trong cay accessibility.
- Issue tracker cua Chromium khong mo duoc qua WebFetch (can dang nhap/JS), nen noi dung bug la tu snippet tim kiem.
- Khong tim thay issue WebView2Feedback khop trieu chung (go phim cham tren DOM lon); ket qua chi la loi chuot/tieu diem lien quan.
