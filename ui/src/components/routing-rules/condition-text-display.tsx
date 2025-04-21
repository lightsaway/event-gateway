import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { ConditionDisplay } from './condition-display';
import { Condition } from '@/services/routing-rules';
import { Badge } from '@/components/ui/badge';

interface ConditionTextDisplayProps {
  condition: Condition;
}

export function ConditionTextDisplay({ condition }: ConditionTextDisplayProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Define colors for different condition types and operators
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

  // Generate a text summary of the condition
  const getConditionText = (condition: Condition): React.ReactNode => {
    // Handle leaf nodes (string expressions)
    if (condition.type && condition.value !== undefined) {
      const type = formatExpressionType(condition.type);
      return (
        <div className="flex items-center gap-1">
          <Badge variant="secondary" className={expressionColors[condition.type as keyof typeof expressionColors]}>
            {type}
          </Badge>
          <span>{condition.value}</span>
        </div>
      );
    }

    // Handle logical operators
    if (condition.and) {
      if (condition.and.length === 0) {
        return (
          <div className="flex items-center gap-1">
            <Badge variant="secondary" className={operatorColors.and}>AND</Badge>
            <span>(empty)</span>
          </div>
        );
      }
      
      if (condition.and.length === 1) {
        return getConditionText(condition.and[0]);
      }
      
      // For multiple conditions, try to summarize the first one
      return (
        <div className="flex items-center gap-1">
          {getConditionText(condition.and[0])}
          <Badge variant="secondary" className={operatorColors.and}>AND</Badge>
          <span>{condition.and.length - 1} more</span>
        </div>
      );
    }

    if (condition.or) {
      if (condition.or.length === 0) {
        return (
          <div className="flex items-center gap-1">
            <Badge variant="secondary" className={operatorColors.or}>OR</Badge>
            <span>(empty)</span>
          </div>
        );
      }
      
      if (condition.or.length === 1) {
        return getConditionText(condition.or[0]);
      }
      
      // For multiple conditions, try to summarize the first one
      return (
        <div className="flex items-center gap-1">
          {getConditionText(condition.or[0])}
          <Badge variant="secondary" className={operatorColors.or}>OR</Badge>
          <span>{condition.or.length - 1} more</span>
        </div>
      );
    }

    if (condition.not) {
      return (
        <div className="flex items-center gap-1">
          <Badge variant="secondary" className={operatorColors.not}>NOT</Badge>
          {getConditionText(condition.not)}
        </div>
      );
    }

    return <span>Empty condition</span>;
  };

  const formatExpressionType = (type: string): string => {
    switch (type) {
      case 'regexMatch': return 'regex';
      case 'startsWith': return 'starts with';
      case 'endsWith': return 'ends with';
      default: return type;
    }
  };

  const conditionText = getConditionText(condition);

  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {isExpanded ? (
            <ChevronDown className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </Button>
        <div className="text-sm flex items-center gap-1">
          {conditionText}
        </div>
      </div>
      {isExpanded && (
        <div className="mt-2 ml-6">
          <ConditionDisplay condition={condition} />
        </div>
      )}
    </div>
  );
} 