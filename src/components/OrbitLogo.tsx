import React from 'react';

export const OrbitLogo: React.FC = () => {
    return (
        <svg className="orbit-logo" viewBox="0 0 240 240" fill="none">
            <defs>
                <linearGradient id="wireGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#ff4800"/>
                    <stop offset="25%" stopColor="#ff6b2b"/>
                    <stop offset="50%" stopColor="#ff8c42"/>
                    <stop offset="75%" stopColor="#ffad5a"/>
                    <stop offset="100%" stopColor="#ffc971"/>
                </linearGradient>
            </defs>

            <g transform="translate(120, 120)">
                {/* Longitude lines */}
                <ellipse cx="0" cy="0" rx="0" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.9"/>
                <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.8"/>
                <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.7"/>
                <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.6"/>
                <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.5"/>
                <ellipse cx="0" cy="0" rx="100" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.8"/>

                {/* Rotated longitude lines */}
                <g transform="rotate(30)">
                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.7"/>
                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.6"/>
                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.5"/>
                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.4"/>
                </g>

                <g transform="rotate(60)">
                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.7"/>
                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.6"/>
                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.5"/>
                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.4"/>
                </g>

                <g transform="rotate(90)">
                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.7"/>
                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.6"/>
                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.5"/>
                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.4"/>
                </g>

                <g transform="rotate(120)">
                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.7"/>
                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.6"/>
                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.5"/>
                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.4"/>
                </g>

                <g transform="rotate(150)">
                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.7"/>
                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.6"/>
                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.5"/>
                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                             strokeWidth="1" fill="none" opacity="0.4"/>
                </g>

                {/* Latitude lines */}
                <ellipse cx="0" cy="0" rx="100" ry="12" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.9"/>
                <ellipse cx="0" cy="-25" rx="90" ry="10" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.8"/>
                <ellipse cx="0" cy="-50" rx="70" ry="8" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.7"/>
                <ellipse cx="0" cy="-75" rx="40" ry="5" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.6"/>
                <ellipse cx="0" cy="-90" rx="15" ry="2" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.5"/>
                <ellipse cx="0" cy="25" rx="90" ry="10" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.8"/>
                <ellipse cx="0" cy="50" rx="70" ry="8" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.7"/>
                <ellipse cx="0" cy="75" rx="40" ry="5" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.6"/>
                <ellipse cx="0" cy="90" rx="15" ry="2" stroke="url(#wireGradient)" strokeWidth="1"
                         fill="none" opacity="0.5"/>

                {/* Poles */}
                <circle cx="0" cy="-100" r="2" fill="url(#wireGradient)" opacity="0.8"/>
                <circle cx="0" cy="100" r="2" fill="url(#wireGradient)" opacity="0.8"/>
            </g>
        </svg>
    );
};