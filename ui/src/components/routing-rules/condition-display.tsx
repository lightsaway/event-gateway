import { Badge } from '@/components/ui/badge';

interface Condition {
  and?: Condition[];
  or?: Condition[];
  not?: Condition;
  type?: string;
  value?: string;
}

const operatorColors = {
  and: 'bg-blue-100 text-blue-800 border-blue-200',
  or: 'bg-green-100 text-green-800 border-green-200',
  not: 'bg-red-100 text-red-800 border-red-200',
} as const;

const expressionColors = {
  regexMatch: 'bg-purple-100 text-purple-800',
  equals: 'bg-gray-100 text-gray-800',
  startsWith: 'bg-yellow-100 text-yellow-800',
  endsWith: 'bg-orange-100 text-orange-800',
  contains: 'bg-pink-100 text-pink-800',
} as const;

function formatExpressionType(type: string): string {
  switch (type) {
    case 'regexMatch': return 'regex';
    case 'startsWith': return 'starts with';
    case 'endsWith': return 'ends with';
    default: return type;
  }
}

interface ConditionDisplayProps {
  condition: Condition;
  level?: number;
}

export function ConditionDisplay({ condition, level = 0 }: ConditionDisplayProps) {
  // Handle leaf nodes (string expressions)
  if (condition.type && condition.value) {
    return (
      <div className="rounded-md border p-2 bg-white shadow-sm">
        <Badge variant="secondary" className={expressionColors[condition.type as keyof typeof expressionColors]}>
          {formatExpressionType(condition.type)}: {condition.value}
        </Badge>
      </div>
    );
  }

  // Handle logical operators
  if (condition.and) {
    return (
      <div className="rounded-md border border-blue-200 bg-white shadow-sm">
        <div className="flex items-center gap-2 p-2 border-b border-blue-200 bg-blue-50 rounded-t-md">
          <Badge variant="secondary" className={operatorColors.and}>AND</Badge>
        </div>
        <div className="p-3 space-y-2">
          {condition.and.map((subCond, index) => (
            <div key={index}>
              <ConditionDisplay condition={subCond} level={level + 1} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (condition.or) {
    return (
      <div className="rounded-md border border-green-200 bg-white shadow-sm">
        <div className="flex items-center gap-2 p-2 border-b border-green-200 bg-green-50 rounded-t-md">
          <Badge variant="secondary" className={operatorColors.or}>OR</Badge>
        </div>
        <div className="p-3 space-y-2">
          {condition.or.map((subCond, index) => (
            <div key={index}>
              <ConditionDisplay condition={subCond} level={level + 1} />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (condition.not) {
    return (
      <div className="rounded-md border border-red-200 bg-white shadow-sm">
        <div className="flex items-center gap-2 p-2 border-b border-red-200 bg-red-50 rounded-t-md">
          <Badge variant="secondary" className={operatorColors.not}>NOT</Badge>
        </div>
        <div className="p-3">
          <ConditionDisplay condition={condition.not} level={level + 1} />
        </div>
      </div>
    );
  }

  return null;
} 