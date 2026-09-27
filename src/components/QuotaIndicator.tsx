import type { Quota } from '../model';
import { resetTime } from '../format';
export function QuotaIndicator({label,quota,now,stale}: {label:string;quota:Quota;now:number;stale:boolean}) {
  const value = quota.remainingPercent;
  const reset = quota.resetsAt === null ? 'Reset unavailable' : `Resets ${new Date(quota.resetsAt * 1000).toLocaleString()}`;
  return <div className={`quota ${stale && value !== null ? 'stale' : ''}`} title={`${label === '5H' ? 'Five-hour' : 'Weekly'} allowance remaining. ${reset}${stale ? '. Last known value; connection unavailable.' : ''}`}>
    <span className="quota-label">{label}</span>
    {value === null ? <span className="unavailable" aria-label={`${label} quota unavailable`}>—</span> : <>
      <span className="percent">{Math.round(value)}%</span><span className="reset">{resetTime(quota.resetsAt, now)}</span>
    </>}
  </div>;
}
