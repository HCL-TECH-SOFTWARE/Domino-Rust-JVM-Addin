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

import java.io.PrintStream;
import java.text.MessageFormat;
import java.util.Optional;
import java.util.concurrent.TimeUnit;
import com.hcl.domino.DominoClient;
import com.hcl.domino.DominoException;
import com.hcl.domino.DominoProcess;
import com.hcl.domino.jna.internal.capi.NotesCAPI;
import com.hcl.domino.mq.MessageQueue;
import com.hcl.domino.server.RunJavaAddin;
import com.hcl.domino.server.ServerStatusLine;

/**
 * Entrypoint used by the Rust native addin to be the main logic of the addin.
 * 
 * <p>This class handles the core message loop of the addin, listening for
 * console commands in the format of "tell rustaddin foo", and then it echoes
 * the text after "tell rustaddin" back to the console.</p>
 * 
 * <p>This class uses JNX's {@code RunJavaAddin} class, but could also use API
 * classes directly, skip API class use, or use {@code JavaServerAddin} from
 * Notes.jar</p>
 */
public class ExampleAddin extends RunJavaAddin {
  private static final PrintStream STDERR = System.err;

  /**
   * Human-readable name of the addin, which shows up in the
   * server task listing and in some log messages.
   */
  public static final String NAME = "Rust Addin";
  /**
   * Programmatic name of the addin, which is used as the message
   * queue name for "tell" commands.
   */
  public static final String ADDIN_NAME = "rustaddin";

  /**
   * This main method creates a new instance of this example addin
   * and joins its thread, effectively delegating control to
   * {@link #runAddin}.
   * 
   * @param args command-line arguments; unused 
   */
  public static void main(String[] args) {
    // Tell JNX to skip NotesInit/NotesTerm, as they're
    // handled in the native side
    System.setProperty("jnx.noinit", "1");
    System.setProperty("jnx.noterm", "1");

    DominoProcess.get().initializeProcess(new String[0]);

    ExampleAddin runner = new ExampleAddin();
    runner.start();
    try {
      runner.join();
    } catch (Throwable e) {
      e.printStackTrace(STDERR);
    } finally {
      DominoProcess.get().terminateProcess();
    }
  }

  public ExampleAddin() {
    super(NAME, ADDIN_NAME);
  }

  public ExampleAddin(String[] args) {
    super(NAME, ADDIN_NAME);
  }

  /**
   * This method is provided by JNX's {@code JavaServerAddin} class and handles the job of
   * providing a {@code DominoClient} object as the current server, as well as creating
   * the server status line and a message queue to listen to.
   */
  @Override
  public void runAddin(DominoClient client, ServerStatusLine status, MessageQueue mq) {

    try {

      PrintStream stdout = System.out;
      PrintStream stderr = System.err;
      try {
        client.getServerAdmin().logMessage(MessageFormat.format("Loading {3} on {0} {1} on {2}",
            System.getProperty("java.runtime.name"),
            System.getProperty("java.runtime.version"), System.getProperty("os.name"), NAME));

        status.setLine("Initializing");

        // Sets the JVM's output to a PrintStream that will use the C API method to log to the console,
        //   which allows it to have a useful prefix and include thread/timestamp information
        System.setOut(new LambdaPrintStream("Rust Addin JVM", s -> log(client, s)));
        System.setErr(new LambdaPrintStream("Rust Addin JVM", s -> log(client, s)));

        client.getServerAdmin().logMessage(String.format("%s: Started", NAME));
        status.setLine("Idle");
        
        // Wait until the message queue is told to quit, i.e. with "tell rustaddin quit"
        while (!mq.isQuitPending()) {
          NotesCAPI.get().OSPreemptOccasionally();

          // This central loop will listen for "tell" commands
          Optional<String> message;
          while ((message = mq.get(5, TimeUnit.SECONDS)) != null) {
            message.ifPresent(msg -> {
              // if we got a message in the queue, process it
              status.setLine("Processing Command");
              
              // This is where a real addin would response to incoming messages
              System.out.println("Asked to do: " + msg);
              
              status.setLine("Idle");
            });
          }
        }
      } finally {
        System.setOut(stdout);
        System.setErr(stderr);
      }
    } catch (InterruptedException e) {
      // Expected - quitting
    } catch (DominoException e) {
      if (e.getId() == 0x0466) {
        // "Quit is pending" = ignore
      } else {
        e.printStackTrace(STDERR);
      }
    } catch (Throwable t) {
      t.printStackTrace(STDERR);
    }
  }

  private void log(DominoClient client, String s) {
    if (s == null) {
      return;
    }
    String msg = format(s);
    if (msg.isEmpty()) {
      return;
    }

    client.getServerAdmin().logMessage(msg);
  }

  private static String format(String s) {
    if (s.isEmpty()) {
      return s;
    }
    int lastGood = s.length();
    for (lastGood = s.length() - 1; lastGood >= 0; lastGood--) {
      if (!Character.isWhitespace(s.charAt(lastGood))) {
        return s.substring(0, lastGood + 1);
      }
    }
    return "";
  }
}
