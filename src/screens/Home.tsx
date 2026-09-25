import React from 'react';
import { HeroBanner } from '../components/HeroBanner';
import { HorizontalRail } from '../components/HorizontalRail';
import { useHomeData } from '../hooks/useAniList';
import { Sparkles } from 'lucide-react';

export const Home: React.FC = () => {
  const { data, isLoading } = useHomeData();

  const heroItems = data?.trending?.slice(0, 5) || [];

  return (
    <div className="flex flex-col gap-2 pb-16 animate-fade-in">
      {/* Hero Highlight Banner */}
      {heroItems.length > 0 && <HeroBanner items={heroItems} />}

      {/* Continue Watching Rail */}
      {data?.continue_watching && data.continue_watching.length > 0 && (
        <HorizontalRail
          title="Continue Watching"
          subtitle="Pick up right where you left off"
          historyItems={data.continue_watching}
          isLoading={isLoading}
        />
      )}

      {/* Trending Rail */}
      <HorizontalRail
        title="Trending This Season"
        subtitle="Most watched anime right now"
        items={data?.trending}
        isLoading={isLoading}
        viewAllLink="/discover?sort=TRENDING_DESC"
      />

      {/* Popular Rail */}
      <HorizontalRail
        title="All-Time Popular"
        subtitle="Fan favorites and legendary series"
        items={data?.popular}
        isLoading={isLoading}
        viewAllLink="/discover?sort=POPULARITY_DESC"
      />

      {/* Top Rated Rail */}
      <HorizontalRail
        title="Highest Rated"
        subtitle="Critically acclaimed masterpieces"
        items={data?.top_rated}
        isLoading={isLoading}
        viewAllLink="/discover?sort=SCORE_DESC"
      />
    </div>
  );
};
