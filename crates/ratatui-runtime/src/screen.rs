use std::fmt;

use crossterm::event::Event;
use ratatui::Frame;

use crate::{App, Runtime};

/// Individual screen in the application.
///
/// # Lifecycle
///
/// Screens go through the following lifecycle:
///
/// 1. **Created** - Screen is constructed
/// 2. **[`on_active`]** - Screen becomes active (foreground)
/// 3. **Active** - Screen handles events, updates, and draws
/// 4. **[`on_inactive`]** - Screen goes to background (Push) or is being removed (Pop/Replace/Exit)
/// 5. **[`on_close`]** - Screen is being removed from stack (Pop/Replace/Exit only)
/// 6. **Dropped** - Screen is destroyed
///
/// ```text
/// Create
///   ↓
/// on_active() ←──────────┐
///   ↓                     │
/// (Active/Foreground)     │ Child screen pops
///   ↓                     │
/// on_inactive()           │
///   ↓                     │
/// (Background) ──────────┘
///   ↓
/// on_close()  ← Only on Pop/Replace/Exit
///   ↓
/// Drop
/// ```
///
/// # Runtime Configuration
///
/// Screens should configure [`Runtime`] settings (tick interval, render mode) in [`on_active`].
/// This ensures the correct settings are applied when the screen becomes active.
///
/// ```rust
/// use crossterm::event::Event;
/// use ratatui::Frame;
/// use ratatui_runtime::{RenderMode, Runtime, Screen, ScreenTransition};
///
/// #[derive(Debug)]
/// struct MyScreen {}
///
/// impl Screen for MyScreen {
///     fn on_active(&mut self, runtime: &mut Runtime) {
///         runtime.set_tick_rate(Some(60.0));
///         runtime.set_render_mode(RenderMode::throttled_from_rate(60.0));
///     }
///
///     fn on_inactive(&mut self, _runtime: &mut Runtime) {}
///     fn on_close(&mut self, _runtime: &mut Runtime) {}
///
///     fn handle_event(&mut self, runtime: &mut Runtime, event: &Event) -> ScreenTransition {
///         ScreenTransition::Stay
///     }
///     fn update(&mut self, _runtime: &mut Runtime) {}
///     fn draw(&self, frame: &mut Frame) {}
/// }
/// ```
///
/// Settings can be changed dynamically in [`handle_event`] or [`update`] if needed.
///
/// [`on_active`]: Self::on_active
/// [`on_inactive`]: Self::on_inactive
/// [`on_close`]: Self::on_close
/// [`handle_event`]: Self::handle_event
/// [`update`]: Self::update
pub trait Screen: fmt::Debug {
    /// Called when this screen becomes active (foreground).
    ///
    /// This is called:
    ///
    /// - On app startup (for the initial screen)
    /// - When this screen is pushed and becomes active
    /// - When popping back to this screen (returning from a child screen)
    ///
    /// Use this to configure [`Runtime`] settings for this screen.
    fn on_active(&mut self, runtime: &mut Runtime);

    /// Called when this screen becomes inactive (background).
    ///
    /// This is called when:
    ///
    /// - Pushing a new screen on top (current screen goes to background, may return later)
    /// - Popping this screen (being removed, [`on_close`] will be called next)
    /// - Replacing this screen (being removed, [`on_close`] will be called next)
    /// - Exiting application (being removed, [`on_close`] will be called next)
    ///
    /// Note: For Pop/Replace/Exit, both [`on_inactive`] and [`on_close`] are called in sequence.
    ///
    /// [`on_close`]: Self::on_close
    /// [`on_inactive`]: Self::on_inactive
    fn on_inactive(&mut self, runtime: &mut Runtime);

    /// Called when this screen is being closed and removed from the stack.
    ///
    /// This is called when:
    ///
    /// - Popping this screen (after [`on_inactive`])
    /// - Replacing this screen (after [`on_inactive`])
    /// - Exiting application (after [`on_inactive`])
    ///
    /// This is NOT called when:
    ///
    /// - Pushing a new screen on top (screen goes to background but stays in stack)
    ///
    /// Use this for cleanup that should only happen when the screen is permanently removed,
    /// such as saving state or releasing resources.
    ///
    /// [`on_inactive`]: Self::on_inactive
    fn on_close(&mut self, runtime: &mut Runtime);

    /// Handles terminal events and returns transition.
    fn handle_event(&mut self, runtime: &mut Runtime, event: &Event) -> ScreenTransition;

    /// Updates screen state (called on each tick).
    fn update(&mut self, runtime: &mut Runtime);

    /// Renders the screen.
    fn draw(&self, frame: &mut Frame);
}

/// Screen transition result from event handling.
#[derive(Debug)]
pub enum ScreenTransition {
    /// Stay in the current screen.
    Stay,

    /// Push a new screen on top of the current one.
    ///
    /// Current screen goes to background (`on_inactive` called).
    /// When the new screen is popped, current screen is reactivated (`on_active` called).
    Push(Box<dyn Screen>),

    /// Pop the current screen and return to the previous one.
    ///
    /// Current screen's `on_inactive` and `on_close` are called,
    /// then previous screen's `on_active` is called.
    Pop,

    /// Replace the current screen with a new one.
    ///
    /// Current screen's `on_inactive` and `on_close` are called,
    /// then new screen's `on_active` is called.
    Replace(Box<dyn Screen>),

    /// Exit the application.
    Exit,
}

/// Screen stack manager that implements App.
#[derive(Debug)]
pub struct ScreenStack<'a> {
    screens: Vec<Box<dyn Screen + 'a>>,
    should_exit: bool,
}

impl<'a> ScreenStack<'a> {
    /// Creates a new screen stack with an initial screen.
    #[must_use]
    pub fn new(initial: Box<dyn Screen + 'a>) -> Self {
        Self {
            screens: vec![initial],
            should_exit: false,
        }
    }

    /// Applies a screen transition.
    fn apply_transition(&mut self, runtime: &mut Runtime, transition: ScreenTransition) {
        match transition {
            ScreenTransition::Stay => {}

            ScreenTransition::Push(mut new_screen) => {
                if let Some(current) = self.screens.last_mut() {
                    current.on_inactive(runtime);
                }
                new_screen.on_active(runtime);
                self.screens.push(new_screen);
            }

            ScreenTransition::Pop => {
                if let Some(mut old_screen) = self.screens.pop() {
                    old_screen.on_inactive(runtime);
                    old_screen.on_close(runtime);
                }
                if let Some(prev_screen) = self.screens.last_mut() {
                    prev_screen.on_active(runtime);
                }
            }

            ScreenTransition::Replace(mut new_screen) => {
                if let Some(mut old_screen) = self.screens.pop() {
                    old_screen.on_inactive(runtime);
                    old_screen.on_close(runtime);
                }
                new_screen.on_active(runtime);
                self.screens.push(new_screen);
            }

            ScreenTransition::Exit => {
                // Clean up all screens
                while let Some(mut screen) = self.screens.pop() {
                    screen.on_inactive(runtime);
                    screen.on_close(runtime);
                }
                self.should_exit = true;
            }
        }
    }
}

impl App for ScreenStack<'_> {
    fn init(&mut self, runtime: &mut Runtime) {
        if let Some(screen) = self.screens.last_mut() {
            screen.on_active(runtime);
        }
    }

    fn should_exit(&self) -> bool {
        self.should_exit || self.screens.is_empty()
    }

    fn handle_event(&mut self, runtime: &mut Runtime, event: Event) {
        if let Some(current) = self.screens.last_mut() {
            let transition = current.handle_event(runtime, &event);
            self.apply_transition(runtime, transition);
        }
    }

    fn draw(&self, frame: &mut Frame) {
        if let Some(current) = self.screens.last() {
            current.draw(frame);
        }
    }

    fn update(&mut self, runtime: &mut Runtime) {
        if let Some(current) = self.screens.last_mut() {
            current.update(runtime);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    /// Tracks lifecycle calls for testing
    #[derive(Debug, Clone, Default)]
    struct LifecycleLog {
        calls: Rc<RefCell<Vec<String>>>,
    }

    impl LifecycleLog {
        fn new() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn log(&self, msg: impl Into<String>) {
            self.calls.borrow_mut().push(msg.into());
        }

        fn get_calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }

        fn clear(&self) {
            self.calls.borrow_mut().clear();
        }
    }

    /// Test screen that logs lifecycle calls
    #[derive(Debug)]
    struct TestScreen {
        name: String,
        log: LifecycleLog,
        transition: ScreenTransition,
    }

    impl TestScreen {
        fn new(name: impl Into<String>, log: LifecycleLog) -> Self {
            Self {
                name: name.into(),
                log,
                transition: ScreenTransition::Stay,
            }
        }

        fn with_transition(mut self, transition: ScreenTransition) -> Self {
            self.transition = transition;
            self
        }
    }

    impl Screen for TestScreen {
        fn on_active(&mut self, _tui: &mut Runtime) {
            self.log.log(format!("{}: on_active", self.name));
        }

        fn on_inactive(&mut self, _tui: &mut Runtime) {
            self.log.log(format!("{}: on_inactive", self.name));
        }

        fn on_close(&mut self, _tui: &mut Runtime) {
            self.log.log(format!("{}: on_close", self.name));
        }

        fn handle_event(&mut self, _tui: &mut Runtime, _event: &Event) -> ScreenTransition {
            self.log.log(format!("{}: handle_event", self.name));
            std::mem::replace(&mut self.transition, ScreenTransition::Stay)
        }

        fn update(&mut self, _tui: &mut Runtime) {
            self.log.log(format!("{}: update", self.name));
        }

        fn draw(&self, _frame: &mut Frame) {
            // No-op for testing
        }
    }

    fn create_test_event() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))
    }

    #[test]
    fn test_init_calls_on_active() {
        let log = LifecycleLog::new();
        let screen = TestScreen::new("A", log.clone());
        let mut stack = ScreenStack::new(Box::new(screen));
        let mut tui = Runtime::new();

        stack.init(&mut tui);

        assert_eq!(log.get_calls(), vec!["A: on_active"]);
    }

    #[test]
    fn test_push_calls_lifecycle() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Push B on top of A
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));

        assert_eq!(
            log.get_calls(),
            vec![
                "A: on_inactive", // A goes to background
                "B: on_active",   // B becomes active
            ]
        );
    }

    #[test]
    fn test_pop_calls_lifecycle() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone()).with_transition(ScreenTransition::Pop);

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        log.clear();

        // Pop B, return to A
        stack.handle_event(&mut tui, create_test_event());

        assert_eq!(
            log.get_calls(),
            vec![
                "B: handle_event", // B handles event
                "B: on_inactive",  // B is being removed
                "B: on_close",     // B is closed
                "A: on_active",    // A is reactivated
            ]
        );
    }

    #[test]
    fn test_replace_calls_lifecycle() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Replace A with B
        stack.apply_transition(&mut tui, ScreenTransition::Replace(Box::new(screen_b)));

        assert_eq!(
            log.get_calls(),
            vec![
                "A: on_inactive", // A is being removed
                "A: on_close",    // A is closed
                "B: on_active",   // B becomes active
            ]
        );
    }

    #[test]
    fn test_exit_calls_lifecycle() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        log.clear();

        // Exit application
        stack.apply_transition(&mut tui, ScreenTransition::Exit);

        assert_eq!(
            log.get_calls(),
            vec![
                "B: on_inactive", // B is being removed
                "B: on_close",    // B is closed
                "A: on_inactive", // A is being removed
                "A: on_close",    // A is closed
            ]
        );
        assert!(stack.should_exit());
    }

    #[test]
    fn test_should_exit_when_empty() {
        let log = LifecycleLog::new();
        let screen = TestScreen::new("A", log.clone()).with_transition(ScreenTransition::Pop);

        let mut stack = ScreenStack::new(Box::new(screen));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        assert!(!stack.should_exit());

        // Pop the last screen
        stack.handle_event(&mut tui, create_test_event());

        assert!(stack.should_exit());
    }

    #[test]
    fn test_update_calls_current_screen() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        log.clear();

        // Update should only call B (current screen)
        stack.update(&mut tui);

        assert_eq!(log.get_calls(), vec!["B: update"]);
    }

    #[test]
    fn test_handle_event_calls_current_screen() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        log.clear();

        // Handle event should only call B (current screen)
        stack.handle_event(&mut tui, create_test_event());

        assert_eq!(log.get_calls(), vec!["B: handle_event"]);
    }

    #[test]
    fn test_nested_push_and_pop() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());
        let screen_c = TestScreen::new("C", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Push B on A
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        // Push C on B
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_c)));

        log.clear();

        // Pop C, return to B
        stack.apply_transition(&mut tui, ScreenTransition::Pop);

        assert_eq!(
            log.get_calls(),
            vec![
                "C: on_inactive", // C is being removed
                "C: on_close",    // C is closed
                "B: on_active",   // B is reactivated
            ]
        );

        log.clear();

        // Pop B, return to A
        stack.apply_transition(&mut tui, ScreenTransition::Pop);

        assert_eq!(
            log.get_calls(),
            vec![
                "B: on_inactive", // B is being removed
                "B: on_close",    // B is closed
                "A: on_active",   // A is reactivated
            ]
        );
    }

    #[test]
    fn test_stay_transition_does_nothing() {
        let log = LifecycleLog::new();
        let screen = TestScreen::new("A", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Stay should not call any lifecycle methods
        stack.apply_transition(&mut tui, ScreenTransition::Stay);

        assert_eq!(log.get_calls(), Vec::<String>::new());
    }

    #[test]
    fn test_push_does_not_call_on_close() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Push B on top of A
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));

        // on_close should NOT be called for A (only on_inactive)
        assert_eq!(
            log.get_calls(),
            vec![
                "A: on_inactive", // A goes to background (no on_close)
                "B: on_active",   // B becomes active
            ]
        );
    }

    #[test]
    fn test_pop_calls_on_close() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        stack.apply_transition(&mut tui, ScreenTransition::Push(Box::new(screen_b)));
        log.clear();

        // Pop B
        stack.apply_transition(&mut tui, ScreenTransition::Pop);

        // on_close should be called for B
        assert_eq!(
            log.get_calls(),
            vec![
                "B: on_inactive", // B is being removed
                "B: on_close",    // B is closed
                "A: on_active",   // A is reactivated
            ]
        );
    }

    #[test]
    fn test_replace_calls_on_close() {
        let log = LifecycleLog::new();
        let screen_a = TestScreen::new("A", log.clone());
        let screen_b = TestScreen::new("B", log.clone());

        let mut stack = ScreenStack::new(Box::new(screen_a));
        let mut tui = Runtime::new();

        stack.init(&mut tui);
        log.clear();

        // Replace A with B
        stack.apply_transition(&mut tui, ScreenTransition::Replace(Box::new(screen_b)));

        // on_close should be called for A
        assert_eq!(
            log.get_calls(),
            vec![
                "A: on_inactive", // A is being removed
                "A: on_close",    // A is closed
                "B: on_active",   // B becomes active
            ]
        );
    }
}
