import React from 'react';
import { Badge } from '@/components/ui/badge';

interface StringExpression {
  type: 'regexMatch' | 'equals' | 'startsWith' | 'endsWith' | 'contains';
  value: string;
}

interface Condition {
  and?: Condition[];
  or?: Condition[];
  not?: Condition;
  type?: string;
  value?: string;
}

const operatorColors = {
  and: 'bg-blue-100 text-blue-800',
  or: 'bg-green-100 text-green-800',
  not: 'bg-red-100 text-red-800',
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
}

export function ConditionDisplay({ condition }: ConditionDisplayProps) {
  // Handle leaf nodes (string expressions)
  if (condition.type && condition.value) {
    return (
      <Badge variant="secondary" className={expressionColors[condition.type as keyof typeof expressionColors]}>
        {formatExpressionType(condition.type)}: {condition.value}
      </Badge>
    );
  }

  // Handle logical operators
  if (condition.and) {
    return (
      <div className="flex flex-wrap gap-1 items-center">
        {condition.and.map((subCond, index) => (
          <React.Fragment key={index}>
            {index > 0 && <Badge variant="secondary" className={operatorColors.and}>AND</Badge>}
            <ConditionDisplay condition={subCond} />
          </React.Fragment>
        ))}
      </div>
    );
  }

  if (condition.or) {
    return (
      <div className="flex flex-wrap gap-1 items-center">
        {condition.or.map((subCond, index) => (
          <React.Fragment key={index}>
            {index > 0 && <Badge variant="secondary" className={operatorColors.or}>OR</Badge>}
            <ConditionDisplay condition={subCond} />
          </React.Fragment>
        ))}
      </div>
    );
  }

  if (condition.not) {
    return (
      <div className="flex flex-wrap gap-1 items-center">
        <Badge variant="secondary" className={operatorColors.not}>NOT</Badge>
        <ConditionDisplay condition={condition.not} />
      </div>
    );
  }

  return null;
} 