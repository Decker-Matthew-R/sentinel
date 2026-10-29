using System.Text;

namespace Sentinel.Agent;

public abstract record VdfNode;
public sealed record VdfString(string Value) : VdfNode;
public sealed record VdfMap(IReadOnlyDictionary<string, VdfNode> Children) : VdfNode;

public static class VdfParser
{
    /// <summary>
    /// Parses a VDF document and returns the root map's children.
    /// The root label (e.g. "AppState") is consumed and discarded.
    /// </summary>
    public static VdfNode Parse(string input)
    {
        var tokens = Tokenize(input);
        var pos = 0;

        if (pos >= tokens.Count)
            throw new FormatException("VDF input is empty");

        if (tokens[pos] is not StringToken)
            throw new FormatException("VDF root must start with a quoted key");
        pos++; // consume root label

        return ParseAfterKey(tokens, ref pos);
    }

    private static VdfNode ParseAfterKey(List<Token> tokens, ref int pos)
    {
        if (pos >= tokens.Count)
            throw new FormatException("Unexpected end of VDF input");

        if (tokens[pos] is OpenBraceToken)
        {
            pos++; // consume '{'
            var children = new Dictionary<string, VdfNode>(StringComparer.OrdinalIgnoreCase);
            while (pos < tokens.Count && tokens[pos] is not CloseBraceToken)
            {
                if (tokens[pos] is not StringToken keyToken)
                    throw new FormatException($"Expected key inside map at token {pos}");
                var key = keyToken.Value;
                pos++;
                children[key] = ParseAfterKey(tokens, ref pos);
            }
            if (pos >= tokens.Count)
                throw new FormatException("Unclosed '{' in VDF input");
            pos++; // consume '}'
            return new VdfMap(children);
        }

        if (tokens[pos] is StringToken valueToken)
        {
            pos++;
            return new VdfString(valueToken.Value);
        }

        throw new FormatException($"Expected value at token position {pos}");
    }

    private static List<Token> Tokenize(string input)
    {
        var tokens = new List<Token>();
        var i = 0;
        while (i < input.Length)
        {
            if (char.IsWhiteSpace(input[i])) { i++; continue; }

            if (input[i] == '/' && i + 1 < input.Length && input[i + 1] == '/')
            {
                while (i < input.Length && input[i] != '\n') i++;
                continue;
            }

            if (input[i] == '{') { tokens.Add(new OpenBraceToken()); i++; continue; }
            if (input[i] == '}') { tokens.Add(new CloseBraceToken()); i++; continue; }

            if (input[i] == '"')
            {
                i++; // skip opening '"'
                var sb = new StringBuilder();
                while (i < input.Length && input[i] != '"')
                {
                    if (input[i] == '\\' && i + 1 < input.Length)
                    {
                        i++; // skip backslash, append escaped char
                        sb.Append(input[i]);
                    }
                    else
                    {
                        sb.Append(input[i]);
                    }
                    i++;
                }
                if (i < input.Length) i++; // skip closing '"'
                tokens.Add(new StringToken(sb.ToString()));
                continue;
            }

            throw new FormatException($"Unexpected character '{input[i]}' at position {i}");
        }
        return tokens;
    }

    private abstract record Token;
    private sealed record StringToken(string Value) : Token;
    private sealed record OpenBraceToken : Token;
    private sealed record CloseBraceToken : Token;
}
