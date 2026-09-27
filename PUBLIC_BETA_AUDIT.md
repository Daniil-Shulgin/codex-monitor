# Codex Monitor — Public Beta audit (ЭТАП 1)

Исторический снимок состояния до ЭТАПА 2. Результаты последующей очистки зафиксированы в `PUBLIC_REPO_CLEANUP.md`; перечисленные ниже временные файлы и compaction-поля могли быть удалены после этого аудита.

Дата аудита: 2026-09-27. Цель: зафиксировать фактический scope `v0.1.0-beta`, без изменения приложения. Статусы ниже различают наличие реализации, автоматическую проверку и наблюдение на живом Windows desktop. «Работает» не означает, что сценарий был повторён вручную в этом аудите; конкретное основание указано в последнем столбце.

## 1. Audited revision и working tree

- `HEAD`: `8754c008c6513f9b3a256344182cbd2b86c25235` (`Initial project`, 2026-09-26 16:12:07 +0200). В commit находится только исходный `README.md`; этот commit **не** содержит проверяемое приложение.
- До создания этого отчёта: `README.md` modified (54 added / 1 removed line), 72 untracked-файла. Изменений staged не было. Ничего не коммичено и не сброшено.
- После аудита единственный дополнительно созданный файл — `PUBLIC_BETA_AUDIT.md`. Таким образом, проверенная версия — **незакоммиченное рабочее дерево**, а не `HEAD` в отдельности. На этапе публикации потребуется зафиксировать именно эти исходники.
- Группы 72 исходных untracked-файлов: `.gitignore`, `CODEX_DATA_SOURCES.md`, `DECISIONS.md`, `PROJECT_STATE.md`, `eslint.config.js`, `index.html`, `package.json`, `package-lock.json`, `tsconfig.json`, `vite.config.ts`; `scripts/` (3 файла); `src/` (14 файлов); `src-tauri/` (включая Rust, конфигурацию, Cargo lock, 2 WAV, icon assets и `gen/schemas/`). Полный перечень на момент аудита воспроизводится командой `git status --porcelain=v1 -uall`; состояние Git намеренно не исправлялось.
- Предыдущий собранный release, **не созданный этим аудитом**: `src-tauri/target/release/codex-monitor.exe`, NSIS `Codex Monitor_0.1.0_x64-setup.exe`, время 2026-09-27 10:09:46, SHA-256 exe `B5F5EB2F3A8E6B73CE85F46897B484D87475981777FB043F17C51BD8C8DD873A`. Оба артефакта есть на диске. Текущая версия в `package.json` и Tauri config — `0.1.0`, beta-метка пока не выставлена.

## 2. Фактическая функциональность

Статус «работает с ограничением» означает либо известное ограничение источника, либо частичное подтверждение. «Не удалось проверить» не означает дефект.

| Сценарий | Статус | Основание и предел проверки |
|---|---|---|
| 5-hour quota | Работает | `cargo run --manifest-path src-tauri/Cargo.toml --bin diagnose` получил значение из локального Codex app-server; `quotas.rs` классифицирует окно по 300 минутам. |
| Weekly quota | Работает | Та же живая диагностика получила недельное значение; классификация по 10080 минутам. |
| Reset time | Работает | Оба `resets_at` пришли в живой диагностике; `QuotaIndicator` показывает относительное время, tooltip — абсолютное. Точное поведение после фактического сброса не воспроизводилось. |
| Collapsed mode | Работает с ограничением | Реализован и покрыт frontend-тестами; прежние версии проверялись вручную. Текущий release в этом аудите нативно не открывался. |
| Expanded mode | Работает с ограничением | Реализован, высота 160 px для пустого списка проверена тестом, предыдущий release собран. Последний вид после исправления нижней рамки нативно не проверен. |
| Task title | Работает с ограничением | Имя читается из SQLite/rollout, нормализуется и ограничивается 160 символами; в UI обрезается с tooltip. На момент живой диагностики задач не было. |
| Running | Работает с ограничением | Явные start-события обрабатываются; синтетические Rust/frontend-тесты проходят. В текущей живой диагностике `running: 0`. |
| Needs action | Работает с ограничением | Явные ошибки/прерывания и доступные blocking-события обрабатываются. Pending approvals другого VS Code app-server не публикуются, поэтому полный охват невозможен. В текущей живой диагностике `action: 0`. |
| Done | Работает с ограничением | Только явные terminal-события, без вывода по бездействию; тесты проходят. В текущей живой диагностике `done: 0`. |
| Elapsed time | Работает с ограничением | Форматирование и остановка таймера для Done проверены тестом. Для terminal Needs action (например, error с `completedAt`) `format.ts` продолжает считать до текущего времени; это следует учитывать при описании таймера. Визуальное обновление на живой задаче сейчас не наблюдалось. |
| Несколько thread/task | Работает с ограничением | `provider` хранит entries по thread ID, lifecycle и UI покрыты синтетическими multi-thread тестами. Одновременные живые VS Code задачи в этом аудите не проверены. |
| Уведомление Needs action | Работает с ограничением | Это **звуковой сигнал**, не системный toast. Логика переходов и gating есть и тестируется; live approval и реальное воспроизведение звука сейчас не проверены. |
| Уведомление Done | Работает с ограничением | Звуковой сигнал на переход Done; защита от исторических/дублирующихся сигналов тестируется. Живой переход сейчас не наблюдался. |
| Звуки | Не удалось проверить | В release встроены два WAV, `rodio` воспроизводит их асинхронно; native audio test помечен `ignored` и требует desktop/audio device. |
| Sound settings | Работает с ограничением | Переключатели Needs action/Done/Enable sound и Volume присутствуют; Rust gating применяет их. Текущее аппаратное воспроизведение не проверено. |
| Dark/Light/System | Работает с ограничением | CSS-переменные и реакция на событие settings есть, theme-switching проверено frontend-тестом и раньше вручную. Последний Support modal в темах нативно не проверен. |
| Opacity | Работает с ограничением | Сохраняемое значение 55–100% применяется к widget CSS; текущий ручной прогон не выполнен. Settings имеет непрозрачный фон по дизайну. |
| Always on Top | Работает с ограничением | Tauri `set_always_on_top`, кнопка и tray action; ранее проверялось, текущий release нативно не открывался. |
| Click-through | Работает с ограничением | Нативное `set_ignore_cursor_events`, глобальная клавиша и tray fallback; ранее переключалось вручную. Текущий release не проверен. |
| Global shortcut | Работает с ограничением | Регистрация при старте, смена с rollback и обработчик; конфликт даёт ошибку и отключает click-through. Зависит от свободной комбинации в системе. |
| Tray fallback | Работает с ограничением | Tray menu возвращает мышь; single/double click вызывает show/unminimize/focus. Последнее поведение на новом release не проверено вручную. |
| Position persistence | Работает с ограничением | `Moved` сохраняет координаты, при старте `clamp` восстанавливает их; ранее проверялось вручную. Текущий restart не проводился. |
| Start with Windows | Не удалось проверить | `tauri-plugin-autostart` подключён, setting и tray action вызывают enable/disable; реальный вход в Windows после включения не проверялся. |
| Settings | Работает с ограничением | Предсозданное скрытое окно, show/focus и native Close→hide; ранее проверялось. Текущая версия после Support не осматривалась нативно. |
| Support modal | Работает с ограничением | Локальный React modal, X/Escape/backdrop, фокус и QR; frontend-тест проверяет открытие/закрытие. Нативные WebView2, Dark/Light и backdrop вручную не проверены. |
| Copy wallet address | Не удалось проверить | `navigator.clipboard.writeText` и inline feedback реализованы; тест использует mock clipboard. Реальное копирование в WebView2 не подтверждено. |
| QR-код | Работает | `qrcode.react` строит SVG локально; независимый `jsqr` декодирует отрисованный SVG ровно в `TGSbmov4GibLQTxLNf9Soju7178Zn1AjXC`. Камерой с экрана не сканировался. |
| Нет quota data | Работает | `remainingPercent:null` даёт `—`; после потери RPC последнее значение остаётся, помечено stale. Проверено frontend и Rust-тестами. |
| Нет активных threads | Работает | Живая диагностика показала 0 задач при подключённом VS Code; expanded UI выводит `Waiting for a Codex turn`. |
| Restart behaviour | Работает с ограничением | Загрузка Settings, позиция и запрет исторических Done реализованы и частично тестируются; текущий release через полный restart не прогонялся. |
| Stale task handling | Работает | После 1 часа без изменения статуса текущий turn скрывается до нового turn; два Rust-теста проверяют переходы. Живое часовое ожидание не проводилось. |

В текущем релизе нет Windows toast/баннеров: слово «notifications» означает только WAV-сигналы для Needs action и Done. UI-счётчики в collapsed mode — количества по **трём статусам**; отдельного общего task count нет.

## 3. Supported — предлагаемый публичный scope `v0.1.0-beta`

- Windows 10/11 x64 с WebView2; локальный Codex extension в VS Code. Текущий аудит выполнялся на Windows x64 `10.0.22000` (Windows 11), Windows 10 отдельно не тестировалась.
- Самостоятельный виджет без открытого терминала; collapsed/expanded, 5H/weekly quota и reset time из локального авторизованного Codex app-server. При недоступном источнике данные отмечаются как unavailable/stale, не подменяются демонстрационными.
- Отслеживание доступных VS Code threads: Running, Needs action, Done, заголовок, elapsed time, несколько записей, скрытие неизменного turn через час. Заявлять Needs action следует с ограничением на approvals (ниже).
- Звуковые сигналы на подтверждённые переходы Needs action/Done, настраиваемые звук и громкость; без Windows toast.
- Settings, темы, прозрачность, Always on Top, click-through с горячей клавишей и tray fallback, сохранение позиции, настройка автозапуска Windows.
- Необязательное окно Support с **публичным** адресом USDT TRC20, локальным QR и кнопкой копирования. Фактическое clipboard-поведение в WebView2 остаётся в перечне ручной проверки, поэтому до неё не стоит давать безусловный claim о копировании.

## 4. Known limitations

1. Другой процесс app-server не видит pending approvals текущего VS Code сервера. Needs action не может обещать универсальное обнаружение approval prompts. Доступны явные error/interruption и те blocking events, которые реально сохраняются.
2. Код и тесты последнего Support modal подтверждены, но native WebView2 UI, реальный clipboard, аппаратный звук и Windows Start-with-Windows не повторены в этом аудите. Computer Use native pipe не подключался на предыдущем этапе; браузерная инвентаризация также была пуста.
3. На момент текущей read-only диагностики IDE connected, но задач 0; live transitions и одновременные threads не подтверждены в этой сессии.
4. Ожидаемая квота может отсутствовать/устареть из-за локального Codex app-server; приложение честно отображает это. Поддержка зависит от формата данных установленной версии Codex extension.
5. Windows 10, другие разрешения/DPI и работа последнего релиза в разных локалях не проверены в этом аудите. Есть прежнее ручное наблюдение Windows desktop 100%, относящееся к более ранней сборке.

## 5. Not included

- macOS, официальное приложение Codex, PyCharm/JetBrains и иные локальные клиенты.
- Automatic compaction count: окончательно удалён из видимого продукта; оставшиеся backend-поля ниже считаются устаревшим кодом, **не** функциональностью beta.
- Windows toast/popup уведомления, общий task count, progress bars quota, fake/demo quota data, blockchain/wallet connection, транзакции, payment processor, backend/telemetry.

## 6. Старые требования и несоответствия

| Где | Находка | Классификация |
|---|---|---|
| `src-tauri/src/model.rs:22`, `src/model.ts:3` | `auto_compactions` / `autoCompactions` остаётся в Task и snapshot-контракте после удаления показателя из UI. | Устаревший продуктовый contract; не виден пользователю. |
| `src-tauri/src/lifecycle.rs:14-15,139-158`, `src-tauri/src/lifecycle.rs:323,373` | Счётчик compaction, deduplication и соответствующие тесты продолжают существовать. | Устаревшая функциональность в backend; не blocker beta. |
| `CODEX_DATA_SOURCES.md`, строка Compaction | Указано «Display `C×—`», хотя UI больше ничего не показывает. | Явно устаревшая документация. |
| `PROJECT_STATE.md` | Исторические 470 px/388 px, старые количества тестов и упоминание прежнего native toast/audio теста находятся рядом с актуальными release-заметками. | История полезна для разработки, но это не описание текущей beta. |
| `README.md` | Нет описания недавно добавленного Support modal; формулировку «notifications» в публичных материалах следует уточнять как звуки. README уже честно говорит, что системных toast нет. | Документационный пробел, не дефект кода. |
| `src/components/Status.tsx`, `src/components/Monitor.tsx` | Три счётчика по статусам остаются в collapsed mode; отдельного общего task count нет. | Текущий UI, **не** устаревший общий счётчик. |

Старых task states в текущей модели нет: только Running/Needs action/Done. Production mock provider, fake/demo quota fallback и старый progress-bar UI не найдены.

## 7. Dead / obsolete / debug / probe code и артефакты

- **Действительно устаревшее для продукта:** перечисленные выше compaction state, сериализуемое поле, обработчик и тесты; сейчас ни один компонент их не читает.
- **Диагностика, не production UI:** `src-tauri/src/bin/diagnose.rs`, `scripts/probe.ps1`, `scripts/inspect-local.cjs`. Первые два документированы в README и дают сводку источников; третий читает локальные схемы и log categories, требует пользовательскую структуру каталогов. `scripts/generate-assets.cjs` — генератор реальных icon/WAV assets, а не fake data provider. `src-tauri/gen/schemas/` — generated Tauri ACL/schema файлы, не runtime source. `.probe/`, `dist/`, `node_modules/`, `src-tauri/target/` исключены `.gitignore`; backup/orig/rej-файлы среди project sources не найдены.
- **Нужный текущий код:** `alerts.rs` (реальные звуки), `diagnostics.rs` (ограниченные журналы статических категорий), `process_job.rs` (закрытие только собственного app-server), `scanner.rs`, `metadata.rs`, `provider.rs`, `rpc.rs`, `runtime.rs`; React-компоненты и Support используются в сборке. Не считать их dead code только по отсутствию прямого UI-импорта.
- **Будущая расширяемость, не текущая feature:** `CodexDataProvider` trait и отделение normalized snapshot от React помогают добавлять providers. Windows-зависимые `rpc.rs`/`process_job.rs` и VS Code фильтры остаются в beta; это не blocker для заявленной Windows + VS Code версии.
- **Тестовые utilities:** `src/test-setup.ts`, `src/*.test.tsx`, Rust `#[cfg(test)]` не являются production UI. `src-tauri/src/bin/diagnose.rs` компилируется как отдельный binary для диагностики, но не запускается основным виджетом. Комментированных экспериментальных реализаций и stale feature flags в production path не найдено. `experimentalApi:true` в `rpc.rs` — действующая capability локального app-server, не неиспользуемый feature flag.

## 8. TODO / FIXME / HACK / TEMP / XXX / незавершённые NOTE

Поиск слов-маркеров в `README.md`, `PROJECT_STATE.md`, `DECISIONS.md`, `CODEX_DATA_SOURCES.md`, `src/`, `src-tauri/src/`, `scripts/`, `package.json` и config не дал записей о незавершённой работе. Совпадения вроде `settings.tmp`, `tempfile`, `template`, `footer-note` не являются TODO-маркерами. Beta blocker по этой категории не найден.

## 9. Test/demo data и достоверность

- В production UI отсутствуют фиксированные quota проценты/времена, демо-задачи и fake fallback. При bootstrap используются null/empty значения `src/model.ts`, которые означают «нет данных» до ответа native. Парсер не придумывает статусы по отсутствию активности.
- Синтетические task IDs, timestamps, названия (`Task`, `Synthetic`, длинный тестовый title), quota проценты/время и lifecycle events находятся в `src/*.test.tsx` и Rust `#[cfg(test)]`. Они не используются основным приложением.
- `TGSbmov4GibLQTxLNf9Soju7178Zn1AjXC` в `src/components/Support.tsx` — заданный публичный receiving address, **не placeholder**. В production source не обнаружены private key, seed phrase, wallet auth, signing, blockchain API или payment processor. QR test проверяет именно этот адрес.
- `scripts/probe.ps1`, `scripts/inspect-local.cjs`, `src-tauri/src/bin/diagnose.rs` — read-only инструменты для разработчика; не создают demo данные в приложении. Локальные диагностические логи `diagnostics.rs` содержат timestamp и статическую категорию, не тексты диалогов/имена задач.

## 10. Документация

- `README.md` в целом описывает Windows + VS Code, ограничения approval visibility, отсутствие toast и fallback квот корректно. Требование Node/Rust/MSVC относится к **сборке из исходников**, не к пользователю готового exe; для installer указаны Windows/WebView2. Windows 10 и автостарт не проверены в этом аудите.
- `README.md` не упоминает Support modal и точный текущий порядок четырёх кнопок footer; это неполнота описания, не ложный claim. Фраза «две кнопки справа» относится к pin/expand и может восприниматься как описание обеих форм виджета.
- `CODEX_DATA_SOURCES.md` содержит устаревшее предложение показывать `C×—`; техническое описание ограничений compaction/approval по-прежнему важно, но не должно превращаться в beta feature claim.
- `PROJECT_STATE.md` — хронология, где старые размеры/число тестов/сборки и устаревший native-toast wording нельзя читать как текущую спецификацию. Последняя запись соответствует текущему артефакту и честно сообщает об отсутствии native visual check.
- `DECISIONS.md` согласован с архитектурой, но пункт про неизвестную причину compaction относится к оставшемуся backend-коду, а не к beta UI.
- Документов, обещающих macOS, Codex desktop app или JetBrains как уже поддерживаемые платформы, не найдено.

## 11. Production-readiness проверки

| Команда | Результат текущего аудита |
|---|---|
| `npm test` | PASS: 16/16 frontend tests, 3 files. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS, 0 предупреждений. |
| `npm run build` | PASS: TypeScript + Vite production build. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS: 24 Rust tests, 1 ignored interactive native audio test, 0 failed. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | PASS. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | PASS. |
| `cargo run --manifest-path src-tauri/Cargo.toml --bin diagnose` | PASS: VS Code detected; обе quota и оба reset timestamp получены; активных tasks 0; отдельный app-server не видит IDE thread (`notLoaded`). |
| `npm run tauri:build` | **Не запускался на ЭТАПЕ 1**: команда создаёт новый exe и NSIS bundle, а задание запрещает создавать release. Предыдущий release с тем же production source уже присутствует и зафиксирован выше; его native UI в этом аудите не проверен. |

Проверки не добавляли зависимости, CI или новые инструменты. Frontend build обновил только ignored `dist/`, Cargo — ignored `target/`. Версия до публикации `v0.1.0-beta` не менялась.

## 12. Реальные release blockers

**Подтверждённых блокеров текущей Windows + VS Code beta не найдено.** Ни один пройденный check не выявил падения основного сценария, записи в источники Codex, утечки приватных данных или неисправной production-сборки. Отсутствие текущей native проверки ряда функций — неопределённость, не доказанный дефект. Незакоммиченный tree и неполная/историческая документация являются предметом ЭТАПА 2 (public repository cleanup), а не основанием рефакторить приложение на ЭТАПЕ 1. Перед публичным утверждением работы Copy address, автозапуска и аппаратного звука нужны честные ручные checks; до проверки их не следует безусловно обещать.

## 13. Итоговый scope `Codex Monitor v0.1.0-beta`

Локальный Windows 10/11 x64 виджет для Codex в VS Code: 5H/weekly quota и reset, три состояния задач с заголовком и таймером, collapsed/expanded UI, звуки событий, Settings, темы/прозрачность, pin/click-through/shortcut/tray, сохранение позиции, опциональный Start with Windows и receive-only Support modal. Ограничение по approvals и непроверенные в текущем аудите нативные сценарии указываются явно. macOS, Codex desktop app, JetBrains и automatic compaction count не входят.

## 14. Вывод этапа

**READY FOR STAGE 2 — public repository cleanup.** Это решение разрешает перейти к чистке публичного репозитория, а не утверждает, что beta уже опубликована или что все native acceptance checks пройдены. Подтверждённых release blockers, которые необходимо устранить до ЭТАПА 2, нет.
