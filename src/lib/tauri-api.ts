import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { exit } from '@tauri-apps/plugin-process';

// Stub logger
const logStub = (name: string) => console.log(`[Stub] ${name} called`);

export const tauriAPI = {
  // --- Window Management ---
  updateContentDimensions: async (dimensions: { width: number; height: number }) => {
    await getCurrentWindow().setSize(new (await import('@tauri-apps/api/window')).LogicalSize(dimensions.width, dimensions.height));
  },
  onToggleExpand: (callback: () => void) => { logStub('onToggleExpand'); return () => { }; },

  // Screenshots (Stubbed for now)
  getScreenshots: async () => [],
  deleteScreenshot: async (path: string) => ({ success: true }),
  onScreenshotTaken: (callback: any) => () => { },
  onScreenshotAttached: (callback: any) => () => { },
  takeScreenshot: async () => { logStub('takeScreenshot'); },

  // Debug/Solutions (Stubbed)
  onSolutionsReady: (cb: any) => () => { },
  onResetView: (cb: any) => () => { },
  onSolutionStart: (cb: any) => () => { },
  onDebugStart: (cb: any) => () => { },
  onDebugSuccess: (cb: any) => () => { },
  onSolutionError: (cb: any) => () => { },
  onProcessingNoScreenshots: (cb: any) => () => { },
  onProblemExtracted: (cb: any) => () => { },
  onSolutionSuccess: (cb: any) => () => { },
  onUnauthorized: (cb: any) => () => { },
  onDebugError: (cb: any) => () => { },

  // Window Movement (Stubbed or verify if needed)
  moveWindowLeft: async () => { },
  moveWindowRight: async () => { },
  moveWindowUp: async () => { },
  moveWindowDown: async () => { },
  toggleWindow: async () => { },
  showWindow: async () => getCurrentWindow().show(),
  hideWindow: async () => getCurrentWindow().hide(),

  // App Lifecycle
  quitApp: async () => { await exit(0); },
  setUndetectable: async (state: boolean) => {
    await getCurrentWindow().setIgnoreCursorEvents(state);
    return { success: true };
  },
  getUndetectable: async () => false,
  setOpenAtLogin: async (open: boolean) => ({ success: true }), // Needs autostart plugin
  getOpenAtLogin: async () => false,

  // Settings Window
  onSettingsVisibilityChange: (cb: any) => () => { },
  toggleSettingsWindow: async (coords?: any) => { },
  closeSettingsWindow: async () => { },
  toggleAdvancedSettings: async () => { },
  closeAdvancedSettings: async () => { },

  // LLM Logic
  getCurrentLlmConfig: async () => ({ provider: 'gemini' as const, model: 'flash', isOllama: false }),
  getAvailableOllamaModels: async () => [],
  switchToOllama: async () => ({ success: false, error: 'Not implemented' }),
  switchToGemini: async () => ({ success: true }),
  testLlmConnection: async () => ({ success: true }),
  selectServiceAccount: async () => ({ success: false, error: 'Not implemented' }),

  // --- Native Audio Service (Core Logic) ---
  onNativeAudioTranscript: (callback: any) => {
    // TODO: Map to Tauri Event
    return listen<any>('transcript', (event) => callback(event.payload)).then(unsub => unsub).catch(console.error) as any;
  },
  // Mapping other events similarly...
  onNativeAudioSuggestion: (callback: any) => {
    listen<any>('suggestion', e => callback(e.payload));
    return () => { }; // Sync return needed by React, but listen is async. We might need a wrapper.
  },
  onNativeAudioConnected: (cb: any) => () => { },
  onNativeAudioDisconnected: (cb: any) => () => { },
  onSuggestionGenerated: (cb: any) => () => { },
  onSuggestionProcessingStart: (cb: any) => () => { },
  onSuggestionError: (cb: any) => () => { },
  generateSuggestion: async (ctx: string, q: string) => invoke('what_should_i_say', { context: ctx }),
  getNativeAudioStatus: async () => ({ connected: true }),

  // Intelligence Mode
  generateAssist: async () => invoke('what_should_i_say'),
  generateWhatToSay: async (q?: string) => invoke('what_should_i_say'),
  generateFollowUp: async () => ({ refined: "Refined text", intent: "follow-up" }),
  generateFollowUpQuestions: async () => ({ questions: "Q1, Q2" }),
  generateRecap: async () => ({ summary: "Summary" }),
  submitManualQuestion: async (q: string) => ({ answer: "Answer", question: q }),
  getIntelligenceContext: async () => ({ context: "", lastAssistantMessage: null, activeMode: "auto" }),
  resetIntelligence: async () => ({ success: true }),

  // Meeting Lifecycle - REAL
  startMeeting: async () => invoke('start_meeting'),
  endMeeting: async () => invoke('stop_meeting'),
  getRecentMeetings: async () => invoke('get_recent_meetings').then((res: any) => res || []),
  getMeetingDetails: async (id: string) => ({}),
  updateMeetingTitle: async () => true,
  updateMeetingSummary: async () => true,
  deleteMeeting: async () => true,
  setWindowMode: async () => { },

  // Events - Intelligence
  onIntelligenceAssistUpdate: (cb: any) => () => { },
  onIntelligenceSuggestedAnswerToken: (cb: any) => () => { },
  onIntelligenceSuggestedAnswer: (cb: any) => () => { },
  onIntelligenceRefinedAnswerToken: (cb: any) => () => { },
  onIntelligenceRefinedAnswer: (cb: any) => () => { },
  onIntelligenceFollowUpQuestionsUpdate: (cb: any) => () => { },
  onIntelligenceFollowUpQuestionsToken: (cb: any) => () => { },
  onIntelligenceRecap: (cb: any) => () => { },
  onIntelligenceRecapToken: (cb: any) => () => { },
  onIntelligenceManualStarted: (cb: any) => () => { },
  onIntelligenceManualResult: (cb: any) => () => { },
  onIntelligenceModeChanged: (cb: any) => () => { },
  onIntelligenceError: (cb: any) => () => { },

  // Streaming
  streamGeminiChat: async () => { },
  onGeminiStreamToken: (cb: any) => () => { },
  onGeminiStreamDone: (cb: any) => () => { },
  onGeminiStreamError: (cb: any) => () => { },

  onMeetingsUpdated: (cb: any) => () => { },

  // Theme
  getThemeMode: async () => ({ mode: 'dark' as const, resolved: 'dark' as const }),
  setThemeMode: async () => { },
  onThemeChanged: (cb: any) => () => { },

  // Calendar
  calendarConnect: async () => ({ success: false }),
  calendarDisconnect: async () => ({ success: true }),
  getCalendarStatus: async () => ({ connected: false }),
  getUpcomingEvents: async () => [],
  calendarRefresh: async () => ({ success: true }),

  // Generic Invoke
  invoke: async (channel: string, ...args: any[]) => {
    // Direct pass-through for anything else
    return invoke(channel, { args });
  },

  // Auto-Update
  onUpdateAvailable: (cb: any) => () => { },
  onUpdateDownloaded: (cb: any) => () => { },
  onUpdateChecking: (cb: any) => () => { },
  onUpdateNotAvailable: (cb: any) => () => { },
  onUpdateError: (cb: any) => () => { },
  restartAndInstall: async () => { },
  checkForUpdates: async () => { },

  // RAG
  ragQueryMeeting: async (id: string, query: string) => invoke('rag_query', { query }),
  ragQueryGlobal: async (query: string) => invoke('rag_query', { query }),
  ragCancelQuery: async () => ({ success: true }),
  ragIsMeetingProcessed: async () => true,
  ragGetQueueStatus: async () => ({ pending: 0, processing: 0, completed: 0, failed: 0 }),
  ragRetryEmbeddings: async () => ({ success: true }),
  onRAGStreamChunk: (cb: any) => () => { },
  onRAGStreamComplete: (cb: any) => () => { },
  onRAGStreamError: (cb: any) => () => { },
};
