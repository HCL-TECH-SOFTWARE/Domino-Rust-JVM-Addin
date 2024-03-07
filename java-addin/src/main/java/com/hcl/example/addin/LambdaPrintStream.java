/*
 * Copyright (c) 2023-2024 HCL America, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.hcl.example.addin;

import java.io.IOException;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.util.Locale;
import java.util.function.Consumer;
import java.util.regex.Pattern;

/**
 * {@link PrintStream} implementation that takes a {@link Consumer} object
 * that is called whenever a new full line should be written to
 * the output, buffering until such a line is ready.
 */
public class LambdaPrintStream extends PrintStream {
  private final String prefix;
  private final Consumer<String> outFunc;

  public LambdaPrintStream(String prefix, Consumer<String> outFunc) {
    super(System.out);
    this.prefix = prefix;
    this.outFunc = outFunc;
  }

  public String getPrefix() {
    return prefix;
  }

  protected void _line(String message) {
    String prefix = getPrefix();
    if (prefix == null || prefix.isEmpty()) {
      outFunc.accept(String.valueOf(message).replace("%", "%%") + '\n'); //$NON-NLS-1$ //$NON-NLS-2$
    } else {
      outFunc.accept(prefix + ": " + String.valueOf(message).replace("%", "%%") + '\n'); //$NON-NLS-1$ //$NON-NLS-2$ //$NON-NLS-3$
    }
  }

  // *******************************************************************************
  // * PrintStream methods
  // *******************************************************************************

  private ThreadLocal<StringBuilder> buffer = ThreadLocal.withInitial(StringBuilder::new);
  private static final Pattern LINE_BREAK = Pattern.compile("[\\r\\n]+"); //$NON-NLS-1$

  @Override
  public void println(String paramString) {
    if (paramString == null || paramString.isEmpty()) {
      println();
      return;
    }

    String message = null;
    if (buffer.get().length() > 0) {
      message = buffer.get().toString() + (paramString == null ? "" : paramString);
      buffer.get().setLength(0);
    } else {
      message = paramString == null ? "" : paramString;
    }

    _lines(message);
  }

  @Override
  public void println() {
    flushBuffer();
  }

  @Override
  public void println(Object paramObject) {
    println(String.valueOf(paramObject));
  }

  @Override
  public void println(boolean paramBoolean) {
    println(String.valueOf(paramBoolean));
  }

  @Override
  public void println(char paramChar) {
    println(String.valueOf(paramChar));
  }

  @Override
  public void println(char[] paramArrayOfChar) {
    println(new String(paramArrayOfChar));
  }

  @Override
  public void println(double paramDouble) {
    println(String.valueOf(paramDouble));
  }

  @Override
  public void println(float paramFloat) {
    println(String.valueOf(paramFloat));
  }

  @Override
  public void println(int paramInt) {
    println(String.valueOf(paramInt));
  }

  @Override
  public void println(long paramLong) {
    println(String.valueOf(paramLong));
  }


  @Override
  public void print(String paramString) {
    if (paramString == null || paramString.isEmpty()) {
      return;
    }

    String[] pieces = LINE_BREAK.split(paramString);
    if (pieces.length > 0) {
      // Print out any whole strings, then append a remainder to the buffer
      for (int i = 0; i < pieces.length - 1; i++) {
        _line(pieces[i]);
      }

      buffer.get().append(pieces[pieces.length - 1]);
    }
  }

  @Override
  public void print(boolean paramBoolean) {
    buffer.get().append(paramBoolean);
  }

  @Override
  public void print(char[] paramArrayOfChar) {
    buffer.get().append(paramArrayOfChar);
  }

  @Override
  public void print(long paramLong) {
    buffer.get().append(paramLong);
  }

  @Override
  public void print(int paramInt) {
    buffer.get().append(paramInt);
  }

  @Override
  public void print(char paramChar) {
    if (paramChar == '\n' || paramChar == '\r') {
      flushBuffer();
    } else {
      buffer.get().append(paramChar);
    }
  }

  @Override
  public void print(double paramDouble) {
    buffer.get().append(paramDouble);
  }

  @Override
  public void print(float paramFloat) {
    buffer.get().append(paramFloat);
  }

  @Override
  public void print(Object paramObject) {
    print(String.valueOf(paramObject));
  }

  @Override
  public PrintStream format(String format, Object... args) {
    print(String.format(format, args));
    return this;
  }

  @Override
  public PrintStream format(Locale l, String format, Object... args) {
    print(String.format(l, format, args));
    return this;
  }

  @Override
  public PrintStream append(char c) {
    buffer.get().append(c);
    return this;
  }

  @Override
  public PrintStream append(CharSequence csq) {
    buffer.get().append(csq);
    return this;
  }

  @Override
  public PrintStream append(CharSequence csq, int start, int end) {
    buffer.get().append(csq, start, end);
    return this;
  }

  @Override
  public PrintStream printf(String format, Object... args) {
    return this.format(format, args);
  }

  @Override
  public PrintStream printf(Locale l, String format, Object... args) {
    return this.format(l, format, args);
  }

  @Override
  public void write(byte[] b) throws IOException {
    buffer.get().append(new String(b, StandardCharsets.UTF_8));
  }

  @Override
  public void write(byte[] buf, int off, int len) {
    buffer.get().append(new String(buf, off, len, StandardCharsets.UTF_8));
  }

  @Override
  public void write(int b) {
    buffer.get().append((char) b);
  }

  /*
   * (non-Javadoc)
   * @see java.io.PrintStream#flush()
   */
  @Override
  public void flush() {
    flushBuffer();
  }

  // *******************************************************************************
  // * Internal implementation methods
  // *******************************************************************************

  private void _lines(String message) {
    String[] pieces = LINE_BREAK.split(message);
    if (pieces.length > 0) {
      for (int i = 0; i < pieces.length; i++) {
        _line(pieces[i]);
      }
    }
  }

  private void flushBuffer() {
    if (buffer.get().length() > 0) {
      _lines(buffer.get().toString());
      buffer.get().setLength(0);
    }
  }
}