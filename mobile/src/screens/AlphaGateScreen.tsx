import React, { useState } from 'react';
import {
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
  StyleSheet,
  Text,
  TextInput,
  TouchableOpacity,
  View,
} from 'react-native';
import { ApiError, claimAlphaInvite } from '../lib/api/client';
import { setStoredConsentGiven, setStoredUserId } from '../lib/storage/secureStore';

interface AlphaGateScreenProps {
  onSuccess: (userId: string) => void;
}

export const AlphaGateScreen: React.FC<AlphaGateScreenProps> = ({ onSuccess }) => {
  const [code, setCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleSubmit = async () => {
    const trimmed = code.trim().toUpperCase();
    if (!trimmed) {
      setErrorMessage("Veuillez saisir votre code d'invitation.");
      return;
    }

    setLoading(true);
    setErrorMessage(null);

    try {
      const res = await claimAlphaInvite(trimmed);
      if (res.claimed && res.user_id) {
        await setStoredUserId(res.user_id);
        await setStoredConsentGiven(true);
        onSuccess(res.user_id);
      } else {
        setErrorMessage(res.message || "Invitation invalide.");
      }
    } catch (err: any) {
      if (err instanceof ApiError) {
        if (err.isInvalidCode) {
          setErrorMessage("Ce code d'invitation est invalide ou a été révoqué.");
        } else if (err.isCodeExpired) {
          setErrorMessage("Ce code d'invitation a expiré ou a atteint sa limite.");
        } else {
          setErrorMessage(err.message);
        }
      } else {
        setErrorMessage("Impossible de joindre le serveur. Vérifiez votre connexion.");
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      style={styles.container}
    >
      <View style={styles.inner}>
        <View style={styles.header}>
          <Text style={styles.title}>Bookfin</Text>
          <Text style={styles.subtitle}>Alpha fermée</Text>
        </View>

        <View style={styles.noticeBox}>
          <Text style={styles.noticeTitle}>À propos de cette version</Text>
          <Text style={styles.noticeBody}>
            Bookfin est une application de lecture littéraire minimaliste.
          </Text>
          <Text style={styles.noticeBody}>
            Pour affiner l'expérience, vos interactions de lecture (préférence Like/Dislike, navigation vers la suite de l'œuvre, temps de lecture actif et défilement) sont mesurées de manière strictement anonyme.
          </Text>
          <Text style={styles.noticeFoot}>
            Aucun compte personnel, aucune donnée nominative, aucun pistage externe.
          </Text>
        </View>

        <View style={styles.form}>
          <Text style={styles.inputLabel}>Code d'accès alpha</Text>
          <TextInput
            style={styles.input}
            placeholder="ALPHA-XXXX"
            placeholderTextColor="#8C8880"
            value={code}
            onChangeText={(text) => {
              setCode(text);
              if (errorMessage) setErrorMessage(null);
            }}
            autoCapitalize="characters"
            autoCorrect={false}
            editable={!loading}
            testID="alpha-code-input"
          />

          {errorMessage && (
            <Text style={styles.errorText} testID="alpha-error-text">
              {errorMessage}
            </Text>
          )}

          <TouchableOpacity
            style={[styles.button, loading && styles.buttonDisabled]}
            onPress={handleSubmit}
            disabled={loading}
            activeOpacity={0.8}
            testID="alpha-submit-button"
          >
            {loading ? (
              <ActivityIndicator color="#F8F6F0" size="small" />
            ) : (
              <Text style={styles.buttonText}>Accepter et commencer la lecture</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F8F6F0',
  },
  inner: {
    flex: 1,
    justifyContent: 'center',
    paddingHorizontal: 28,
    maxWidth: 500,
    width: '100%',
    alignSelf: 'center',
  },
  header: {
    marginBottom: 32,
    alignItems: 'center',
  },
  title: {
    fontFamily: Platform.OS === 'ios' ? 'Georgia' : 'serif',
    fontSize: 34,
    fontWeight: '600',
    color: '#1F1E1C',
    letterSpacing: 0.5,
  },
  subtitle: {
    fontFamily: Platform.OS === 'ios' ? 'Georgia' : 'serif',
    fontSize: 14,
    color: '#736F66',
    marginTop: 4,
    textTransform: 'uppercase',
    letterSpacing: 1.5,
  },
  noticeBox: {
    backgroundColor: '#EFECE4',
    borderRadius: 8,
    padding: 18,
    marginBottom: 28,
    borderWidth: 1,
    borderColor: '#E2DED4',
  },
  noticeTitle: {
    fontFamily: Platform.OS === 'ios' ? 'Georgia' : 'serif',
    fontSize: 15,
    fontWeight: '600',
    color: '#2A2927',
    marginBottom: 8,
  },
  noticeBody: {
    fontSize: 13,
    lineHeight: 19,
    color: '#4A4843',
    marginBottom: 8,
  },
  noticeFoot: {
    fontSize: 12,
    lineHeight: 17,
    color: '#736F66',
    fontStyle: 'italic',
  },
  form: {
    width: '100%',
  },
  inputLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#4A4843',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#FFFFFF',
    borderWidth: 1,
    borderColor: '#D4CEBF',
    borderRadius: 6,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1F1E1C',
    fontFamily: Platform.OS === 'ios' ? 'Courier' : 'monospace',
    letterSpacing: 1,
  },
  errorText: {
    color: '#A82B1E',
    fontSize: 13,
    marginTop: 8,
    lineHeight: 18,
  },
  button: {
    backgroundColor: '#1F1E1C',
    borderRadius: 6,
    paddingVertical: 14,
    alignItems: 'center',
    marginTop: 20,
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  buttonText: {
    color: '#F8F6F0',
    fontSize: 15,
    fontWeight: '600',
    letterSpacing: 0.5,
  },
});

