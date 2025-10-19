import React from 'react'
import { Link } from 'react-router-dom'

export const LandingPage: React.FC = () => {
  return (
    <div className="min-h-screen" style={{ 
      background: 'linear-gradient(135deg, var(--bg-darker) 0%, var(--bg-dark) 100%)' 
    }}>
      {/* Hero Section */}
      <div style={{ 
        padding: '2rem 1rem', 
        maxWidth: '1200px', 
        margin: '0 auto',
        textAlign: 'center' as const
      }}>
        {/* Logo and Title */}
        <div style={{ marginBottom: '3rem' }}>
          <div style={{ 
            width: '80px', 
            height: '80px', 
            margin: '0 auto 1.5rem', 
            borderRadius: '20px', 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center',
            background: 'linear-gradient(135deg, var(--bitcoin-orange) 0%, var(--bitcoin-dark) 100%)',
            boxShadow: '0 10px 30px rgba(247, 147, 26, 0.3)'
          }}>
            <svg width="40" height="40" viewBox="0 0 24 24" fill="white">
              <path d="M12 2C13.1 2 14 2.9 14 4V6.28C15.6 6.68 16.95 7.75 17.65 9.19L16.26 9.81C15.8 8.8 14.9 8.13 13.85 7.93L14 8H14.85C15.5 8 16 8.5 16 9.15V9.85C16 10.5 15.5 11 14.85 11H13V13H14.85C15.5 13 16 13.5 16 14.15V14.85C16 15.5 15.5 16 14.85 16H14L13.85 16.07C14.9 15.87 15.8 15.2 16.26 14.19L17.65 14.81C16.95 16.25 15.6 17.32 14 17.72V20C14 21.1 13.1 22 12 22S10 21.1 10 20V17.72C8.4 17.32 7.05 16.25 6.35 14.81L7.74 14.19C8.2 15.2 9.1 15.87 10.15 16.07L10 16H9.15C8.5 16 8 15.5 8 14.85V14.15C8 13.5 8.5 13 9.15 13H11V11H9.15C8.5 11 8 10.5 8 9.85V9.15C8 8.5 8.5 8 9.15 8H10L10.15 7.93C9.1 8.13 8.2 8.8 7.74 9.81L6.35 9.19C7.05 7.75 8.4 6.68 10 6.28V4C10 2.9 10.9 2 12 2M11 9V11H13V9H11M11 13V15H13V13H11Z"/>
            </svg>
          </div>
          
          <h1 style={{ 
            fontSize: 'clamp(2.5rem, 5vw, 4rem)', 
            fontWeight: '800', 
            color: 'var(--text-primary)',
            marginBottom: '1rem',
            background: 'linear-gradient(135deg, var(--bitcoin-orange), var(--bitcoin-light))',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            backgroundClip: 'text'
          }}>
            PesaBit
          </h1>
          
          <p style={{ 
            fontSize: '1.25rem', 
            color: 'var(--text-secondary)', 
            maxWidth: '600px', 
            margin: '0 auto 2rem',
            lineHeight: '1.6'
          }}>
            Bridge your M-Pesa to Bitcoin Lightning Network. Send money globally in seconds, not days.
          </p>
          
          {/* CTA Buttons */}
          <div style={{ 
            display: 'flex', 
            gap: '1rem', 
            justifyContent: 'center',
            flexWrap: 'wrap' as const,
            marginBottom: '3rem'
          }}>
            <Link 
              to="/register" 
              className="btn-primary"
              style={{ 
                padding: '1rem 2rem', 
                fontSize: '1.1rem',
                textDecoration: 'none',
                display: 'inline-block',
                minWidth: '160px'
              }}
            >
              Start Trading →
            </Link>
            <Link 
              to="/login" 
              className="btn-secondary"
              style={{ 
                padding: '1rem 2rem', 
                fontSize: '1.1rem',
                textDecoration: 'none',
                display: 'inline-block',
                minWidth: '160px'
              }}
            >
              Sign In
            </Link>
          </div>
        </div>

        {/* Features Grid */}
        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', 
          gap: '1.5rem',
          marginBottom: '4rem'
        }}>
          {/* Feature 1 */}
          <div className="card" style={{ textAlign: 'left' as const }}>
            <div style={{ 
              width: '50px', 
              height: '50px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--bitcoin-orange), var(--bitcoin-light))',
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              marginBottom: '1rem'
            }}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
                <path d="M13,9H18.5L13,3.5V9M6,2H14L20,8V20A2,2 0 0,1 18,22H6C4.89,22 4,21.1 4,20V4C4,2.89 4.89,2 6,2M15,18V16H6V18H15M18,14V12H6V14H18Z"/>
              </svg>
            </div>
            <h3 style={{ 
              fontSize: '1.25rem', 
              fontWeight: '600', 
              color: 'var(--text-primary)', 
              marginBottom: '0.5rem' 
            }}>Instant Transfers</h3>
            <p style={{ color: 'var(--text-secondary)', lineHeight: '1.6' }}>
              Lightning Network enables sub-second Bitcoin transfers with minimal fees
            </p>
          </div>

          {/* Feature 2 */}
          <div className="card" style={{ textAlign: 'left' as const }}>
            <div style={{ 
              width: '50px', 
              height: '50px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--green-success), var(--green-light))',
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              marginBottom: '1rem'
            }}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
                <path d="M12,1L3,5V11C3,16.55 6.84,21.74 12,23C17.16,21.74 21,16.55 21,11V5L12,1M10,17L6,13L7.41,11.59L10,14.17L16.59,7.58L18,9L10,17Z"/>
              </svg>
            </div>
            <h3 style={{ 
              fontSize: '1.25rem', 
              fontWeight: '600', 
              color: 'var(--text-primary)', 
              marginBottom: '0.5rem' 
            }}>Bank-Grade Security</h3>
            <p style={{ color: 'var(--text-secondary)', lineHeight: '1.6' }}>
              Multi-signature wallets and encrypted keys protect your funds 24/7
            </p>
          </div>

          {/* Feature 3 */}
          <div className="card" style={{ textAlign: 'left' as const }}>
            <div style={{ 
              width: '50px', 
              height: '50px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--blue-accent), #3b82f6)',
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              marginBottom: '1rem'
            }}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="white">
                <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M7.07,18.28C7.5,17.38 10.12,16.5 12,16.5C13.88,16.5 16.5,17.38 16.93,18.28C15.57,19.36 13.86,20 12,20C10.14,20 8.43,19.36 7.07,18.28M18.36,16.83C16.93,15.09 13.46,14.5 12,14.5C10.54,14.5 7.07,15.09 5.64,16.83C4.62,15.5 4,13.82 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,13.82 19.38,15.5 18.36,16.83M12,6C10.06,6 8.5,7.56 8.5,9.5C8.5,11.44 10.06,13 12,13C13.94,13 15.5,11.44 15.5,9.5C15.5,7.56 13.94,6 12,6M12,11A1.5,1.5 0 0,1 10.5,9.5A1.5,1.5 0 0,1 12,8A1.5,1.5 0 0,1 13.5,9.5A1.5,1.5 0 0,1 12,11Z"/>
              </svg>
            </div>
            <h3 style={{ 
              fontSize: '1.25rem', 
              fontWeight: '600', 
              color: 'var(--text-primary)', 
              marginBottom: '0.5rem' 
            }}>Simple Interface</h3>
            <p style={{ color: 'var(--text-secondary)', lineHeight: '1.6' }}>
              Familiar M-Pesa experience makes Bitcoin accessible to everyone
            </p>
          </div>
        </div>

        {/* Stats Section */}
        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', 
          gap: '2rem',
          marginBottom: '4rem',
          padding: '2rem',
          background: 'rgba(22, 27, 34, 0.5)',
          borderRadius: '16px',
          border: '1px solid var(--border-default)'
        }}>
          <div style={{ textAlign: 'center' as const }}>
            <div style={{ 
              fontSize: '2.5rem', 
              fontWeight: '800', 
              color: 'var(--bitcoin-orange)',
              marginBottom: '0.5rem'
            }}>$2.1M+</div>
            <p style={{ color: 'var(--text-secondary)' }}>Volume Traded</p>
          </div>
          <div style={{ textAlign: 'center' as const }}>
            <div style={{ 
              fontSize: '2.5rem', 
              fontWeight: '800', 
              color: 'var(--bitcoin-orange)',
              marginBottom: '0.5rem'
            }}>15K+</div>
            <p style={{ color: 'var(--text-secondary)' }}>Active Users</p>
          </div>
          <div style={{ textAlign: 'center' as const }}>
            <div style={{ 
              fontSize: '2.5rem', 
              fontWeight: '800', 
              color: 'var(--bitcoin-orange)',
              marginBottom: '0.5rem'
            }}>&lt;2s</div>
            <p style={{ color: 'var(--text-secondary)' }}>Avg Transfer Time</p>
          </div>
        </div>

        {/* Final CTA */}
        <div style={{ 
          textAlign: 'center' as const,
          padding: '2rem',
          background: 'linear-gradient(135deg, rgba(247, 147, 26, 0.1), rgba(204, 122, 0, 0.1))',
          borderRadius: '16px',
          border: '1px solid rgba(247, 147, 26, 0.2)'
        }}>
          <h2 style={{ 
            fontSize: '1.75rem', 
            fontWeight: '700', 
            color: 'var(--text-primary)', 
            marginBottom: '1rem' 
          }}>
            Ready to bridge M-Pesa and Bitcoin?
          </h2>
          <p style={{ 
            color: 'var(--text-secondary)', 
            marginBottom: '1.5rem',
            fontSize: '1.1rem'
          }}>
            Join thousands of users already trading globally
          </p>
          <Link 
            to="/register" 
            className="btn-primary"
            style={{ 
              padding: '1rem 2.5rem', 
              fontSize: '1.1rem',
              textDecoration: 'none',
              display: 'inline-block'
            }}
          >
            Get Started Free →
          </Link>
        </div>
      </div>
    </div>
  )
}