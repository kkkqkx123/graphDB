import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import SpaceList from './SpaceList';

const Schema: React.FC = () => {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="spaces" replace />} />
      <Route path="spaces" element={<SpaceList />} />
      <Route path="tags" element={<div>Tag Management - Coming soon...</div>} />
      <Route path="edges" element={<div>Edge Management - Coming soon...</div>} />
      <Route path="indexes" element={<div>Index Management - Coming soon...</div>} />
    </Routes>
  );
};

export default Schema;
