import { useEffect, useRef, useState, type KeyboardEvent, type MouseEvent } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { X } from 'lucide-react';

export const SUPPORT_ADDRESS = 'TGSbmov4GibLQTxLNf9Soju7178Zn1AjXC';

export function Support() {
  const [open, setOpen] = useState(false);
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'failed'>('idle');
  const trigger = useRef<HTMLButtonElement>(null);
  const close = useRef<HTMLButtonElement>(null);
  const dialog = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const triggerButton = trigger.current;
    close.current?.focus();
    return () => triggerButton?.focus();
  }, [open]);

  useEffect(() => {
    if (copyState === 'idle') return;
    const timer = window.setTimeout(() => setCopyState('idle'), 2200);
    return () => window.clearTimeout(timer);
  }, [copyState]);

  function onDialogKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      setOpen(false);
      return;
    }
    if (event.key !== 'Tab' || !dialog.current) return;
    const controls = Array.from(dialog.current.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'));
    if (controls.length === 0) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function onBackdropClick(event: MouseEvent<HTMLDivElement>) {
    if (event.target === event.currentTarget) setOpen(false);
  }

  async function copyAddress() {
    try {
      await navigator.clipboard.writeText(SUPPORT_ADDRESS);
      setCopyState('copied');
    } catch {
      setCopyState('failed');
    }
  }

  return <>
    <div className="support-footer">
      <button ref={trigger} type="button" className="support-trigger" onClick={() => setOpen(true)}>Support Codex Monitor</button>
      <p>If the app is useful to you, you can optionally support its development.</p>
    </div>
    {open && <div className="support-backdrop" onMouseDown={onBackdropClick}>
      <div ref={dialog} className="support-dialog" role="dialog" aria-modal="true" aria-labelledby="support-title" aria-describedby="support-description" onKeyDown={onDialogKeyDown}>
        <div className="support-heading"><h2 id="support-title">Support Codex Monitor</h2><button ref={close} type="button" className="support-close" aria-label="Close support" onClick={() => setOpen(false)}><X size={15}/></button></div>
        <p id="support-description">If the app is useful to you, you can optionally support its development.</p>
        <div className="support-network">USDT · TRC20 (TRON)</div>
        <div className="support-qr"><QRCodeSVG value={SUPPORT_ADDRESS} size={152} level="M" marginSize={4} title="USDT TRC20 receiving address"/></div>
        <p className="support-warning">Send only USDT via TRC20 to this address.</p>
        <div className="support-address-row"><code>{SUPPORT_ADDRESS}</code><button type="button" className="support-copy" onClick={() => void copyAddress()}>Copy address</button></div>
        <span className="support-copy-feedback" role="status">{copyState === 'copied' ? 'Copied' : copyState === 'failed' ? 'Could not copy' : '\u00a0'}</span>
      </div>
    </div>}
  </>;
}
